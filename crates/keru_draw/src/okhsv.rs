//! CPU side OKHSV, matching the GPU math in `shaders/keru_draw.slang`.
//! See https://bottosson.github.io/posts/colorpicker/ and https://bottosson.github.io/posts/gamutclipping/

use crate::Color;

fn oklab_to_linear_srgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    (
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    )
}

fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.0031308 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 }
}

fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

fn linear_srgb_to_oklab(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    (
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    )
}

fn toe(x: f32) -> f32 {
    const K_1: f32 = 0.206;
    const K_2: f32 = 0.03;
    const K_3: f32 = (1.0 + K_1) / (1.0 + K_2);
    0.5 * (K_3 * x - K_1 + ((K_3 * x - K_1) * (K_3 * x - K_1) + 4.0 * K_2 * K_3 * x).sqrt())
}

fn toe_inv(x: f32) -> f32 {
    const K_1: f32 = 0.206;
    const K_2: f32 = 0.03;
    const K_3: f32 = (1.0 + K_1) / (1.0 + K_2);
    (x * x + K_1 * x) / (K_3 * (x + K_2))
}

/// The gamut triangle's (S, T) bounds at the cusp `(l_cusp, c_cusp)`.
fn to_st(l_cusp: f32, c_cusp: f32) -> (f32, f32) {
    (c_cusp / l_cusp, c_cusp / (1.0 - l_cusp))
}

/// Maximum saturation (S = C/L) possible for a given (normalized) hue before the sRGB gamut is exceeded.
fn compute_max_saturation(a: f32, b: f32) -> f32 {
    let (k0, k1, k2, k3, k4, wl, wm, ws);
    if -1.88170328 * a - 0.80936493 * b > 1.0 {
        (k0, k1, k2, k3, k4) = (1.19086277, 1.76576728, 0.59662641, 0.75515197, 0.56771245);
        (wl, wm, ws) = (4.0767416621, -3.3077115913, 0.2309699292);
    } else if 1.81444104 * a - 1.19445276 * b > 1.0 {
        (k0, k1, k2, k3, k4) = (0.73956515, -0.45954404, 0.08285427, 0.12541070, 0.14503204);
        (wl, wm, ws) = (-1.2684380046, 2.6097574011, -0.3413193965);
    } else {
        (k0, k1, k2, k3, k4) = (1.35733652, -0.00915799, -1.15130210, -0.50559606, 0.00692167);
        (wl, wm, ws) = (-0.0041960863, -0.7034186147, 1.7076147010);
    }

    let mut s = k0 + k1 * a + k2 * b + k3 * a * a + k4 * a * b;

    let k_l = 0.3963377774 * a + 0.2158037573 * b;
    let k_m = -0.1055613458 * a - 0.0638541728 * b;
    let k_s = -0.0894841775 * a - 1.2914855480 * b;

    let l_ = 1.0 + s * k_l;
    let m_ = 1.0 + s * k_m;
    let s_ = 1.0 + s * k_s;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let l_ds = 3.0 * k_l * l_ * l_;
    let m_ds = 3.0 * k_m * m_ * m_;
    let s_ds = 3.0 * k_s * s_ * s_;

    let l_ds2 = 6.0 * k_l * k_l * l_;
    let m_ds2 = 6.0 * k_m * k_m * m_;
    let s_ds2 = 6.0 * k_s * k_s * s_;

    let f = wl * l + wm * m + ws * s3;
    let f1 = wl * l_ds + wm * m_ds + ws * s_ds;
    let f2 = wl * l_ds2 + wm * m_ds2 + ws * s_ds2;

    s -= f * f1 / (f1 * f1 - 0.5 * f * f2);
    s
}

/// The cusp of the sRGB gamut's triangle-like shape in Oklab, for a given (normalized) hue: (L, C).
/// `a`/`b` must be cos/sin of the hue angle, so `a*a + b*b == 1`.
fn find_cusp(a: f32, b: f32) -> (f32, f32) {
    let s_cusp = compute_max_saturation(a, b);
    let (r, g, bl) = oklab_to_linear_srgb(1.0, s_cusp * a, s_cusp * b);
    let l_cusp = (1.0 / r.max(g).max(bl)).cbrt();
    let c_cusp = l_cusp * s_cusp;
    (l_cusp, c_cusp)
}

/// Fixed-hue OKHSV (saturation, value in `[0, 1]`) to a displayable [`Color`]. `hue` is in `[0, 1)`.
/// Same result as what [`crate::OkhsvSaturationValueGradient`]/[`crate::OkhsvHueGradient`] draw on screen.
pub fn okhsv_to_srgb(hue: f32, saturation: f32, value: f32) -> Color {
    let a_ = (2.0 * std::f32::consts::PI * hue).cos();
    let b_ = (2.0 * std::f32::consts::PI * hue).sin();

    let (l_cusp, c_cusp) = find_cusp(a_, b_);
    let (s_max, t_max) = to_st(l_cusp, c_cusp);
    let s_0 = 0.5;
    let k = 1.0 - s_0 / s_max;

    let l_v = 1.0 - saturation * s_0 / (s_0 + t_max - t_max * k * saturation);
    let c_v = saturation * t_max * s_0 / (s_0 + t_max - t_max * k * saturation);

    let l = value * l_v;
    let mut c = value * c_v;

    let l_vt = toe_inv(l_v);
    let c_vt = c_v * l_vt / l_v;
    let l_new = toe_inv(l);
    c = c * l_new / l.max(1e-6);
    let mut l = l_new;

    let (sr, sg, sb) = oklab_to_linear_srgb(l_vt, a_ * c_vt, b_ * c_vt);
    let scale_l = (1.0 / sr.max(sg).max(sb).max(0.0)).cbrt();
    l *= scale_l;
    c *= scale_l;

    let (r, g, b) = oklab_to_linear_srgb(l, c * a_, c * b_);
    Color::new(
        linear_to_srgb(r.clamp(0.0, 1.0)),
        linear_to_srgb(g.clamp(0.0, 1.0)),
        linear_to_srgb(b.clamp(0.0, 1.0)),
        1.0,
    )
}

/// The exact inverse of [`okhsv_to_srgb`]: `(hue, saturation, value)`, each in `[0, 1]` (hue in `[0, 1)`).
/// For a fully achromatic color (grey, black or white), hue is meaningless; `0.0` is returned for it.
pub fn srgb_to_okhsv(color: Color) -> (f32, f32, f32) {
    let (lr, lg, lb) = (srgb_to_linear(color.r), srgb_to_linear(color.g), srgb_to_linear(color.b));
    let (l, lab_a, lab_b) = linear_srgb_to_oklab(lr, lg, lb);

    let c = (lab_a * lab_a + lab_b * lab_b).sqrt();
    if c < 1e-5 {
        return (0.0, 0.0, toe(l));
    }
    let a_ = lab_a / c;
    let b_ = lab_b / c;
    let hue = (0.5 + 0.5 * (-lab_b).atan2(-lab_a) / std::f32::consts::PI).rem_euclid(1.0);

    let (l_cusp, c_cusp) = find_cusp(a_, b_);
    let (s_max, t_max) = to_st(l_cusp, c_cusp);
    let s_0 = 0.5;
    let k = 1.0 - s_0 / s_max;

    let t = t_max / (c + l * t_max);
    let l_v = t * l;
    let c_v = t * c;
    let l_vt = toe_inv(l_v);
    let c_vt = c_v * l_vt / l_v;

    let (sr, sg, sb) = oklab_to_linear_srgb(l_vt, a_ * c_vt, b_ * c_vt);
    let scale_l = (1.0 / sr.max(sg).max(sb).max(0.0)).cbrt();
    let l = l / scale_l;

    let l = toe(l);

    let v = l / l_v;
    let s = (s_0 + t_max) * c_v / (t_max * s_0 + t_max * k * c_v);

    (hue, s.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for i in 0..7 {
            let hue = i as f32 / 7.0;
            for s10 in 1..10 {
                for v10 in 1..10 {
                    let (s, v) = (s10 as f32 / 10.0, v10 as f32 / 10.0);
                    let color = okhsv_to_srgb(hue, s, v);
                    let (h2, s2, v2) = srgb_to_okhsv(color);
                    let color2 = okhsv_to_srgb(h2, s2, v2);
                    assert!(
                        (color.r - color2.r).abs() < 0.01 && (color.g - color2.g).abs() < 0.01 && (color.b - color2.b).abs() < 0.01,
                        "hue={hue} s={s} v={v}: {color:?} -> ({h2}, {s2}, {v2}) -> {color2:?}"
                    );
                }
            }
        }
    }
}
