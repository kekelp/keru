#[derive(Copy, Clone, Debug)]
pub(crate) struct OkLchColor {
    pub(crate) lightness: f32,
    pub(crate) hue: f32,
    pub(crate) chroma: f32,
}
impl Into<[f32; 3]> for OkLchColor {
    fn into(self) -> [f32; 3] {
        return [self.hue, self.chroma, self.lightness];
    }
}

struct OkLabColor {
    l: f32,
    a: f32,
    b: f32,
}

struct RgbColor {
    r: f32,
    g: f32,
    b: f32,
}

fn linear_srgb_to_oklab(c: RgbColor) -> OkLabColor {
    let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    OkLabColor {
        l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    }
}

fn oklab_to_linear_srgb(c: OkLabColor) -> RgbColor {
    let l_ = c.l + 0.3963377774 * c.a + 0.2158037573 * c.b;
    let m_ = c.l - 0.1055613458 * c.a - 0.0638541728 * c.b;
    let s_ = c.l - 0.0894841775 * c.a - 1.2914855480 * c.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    RgbColor {
        r: 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        g: -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        b: -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    }
}
