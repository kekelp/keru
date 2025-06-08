use bytemuck::{Pod, Zeroable};
use glam::{vec2, Vec2};
use std::{hash::{Hash, Hasher}, ops::{Add, Div, Index, IndexMut, Mul, Sub}};
use Axis::*;

use crate::*;

/// A length on the screen, expressed either as pixels or as a fraction of a parent rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Len {
    Pixels(u32),
    Frac(f32),
}
impl Len {
    pub const ZERO: Self = Self::Pixels(0);
}
impl Hash for Len {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Len::Pixels(p) => (0u8, p).hash(state),
            Len::Frac(f) => (1u8, f.to_bits()).hash(state),
        }
    }
}

impl Ui {
    pub(crate) fn f32_size_to_pixels2(&self, size: Xy<f32>) -> Xy<u32> {
        return Xy::new(
            (size.x * self.sys.unifs.size[X]) as u32,
            (size.y * self.sys.unifs.size[Y]) as u32
        );
    }

    pub(crate) fn pixels_to_frac(&self, pixels: u32, axis: Axis) -> f32 {
        return (pixels as f32) / self.sys.unifs.size[axis];
    }
    pub(crate) fn pixels_to_frac2(&self, pixels: Xy<u32>) -> Xy<f32> {
        return Xy::new(
            self.pixels_to_frac(pixels.x, X),
            self.pixels_to_frac(pixels.y, Y),
        );
    }

    pub(crate) fn f32_pixels_to_frac(&self, pixels: f32, axis: Axis) -> f32 {
        return pixels / self.sys.unifs.size[axis];
    }

    pub(crate) fn f32_pixels_to_frac2(&self, pixels: Xy<f32>) -> Xy<f32> {
        return Xy::new(
            self.f32_pixels_to_frac(pixels.x, X),
            self.f32_pixels_to_frac(pixels.y, Y),
        );
    }

    pub(crate) fn len_to_frac_of_size(&self, len: Len, parent: Xy<f32>, axis: Axis) -> f32 {
        match len {
            Len::Pixels(pixels) => {
                return self.pixels_to_frac(pixels, axis);
            },
            Len::Frac(frac) => {
                return parent[axis] * frac;
            }
        };
    }
}

/// The X or Y axes.
#[derive(Debug, Clone, Copy, Hash)]
pub enum Axis {
    X,
    Y,
}
impl Axis {
    pub fn other(&self) -> Self {
        match self {
            Axis::X => return Axis::Y,
            Axis::Y => return Axis::X,
        }
    }
}


/// A generic container for two-dimensional data.
///
/// Used in [`Layout`], [`RenderInfo`] and other places.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Xy<T> {
    pub x: T,
    pub y: T,
}

impl<T: Hash> Hash for Xy<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl<T: Add<Output = T> + Copy> Add<Xy<T>> for Xy<T> {
    type Output = Self;
    fn add(self, rhs: Xy<T>) -> Self::Output {
        let new_x = self.x + rhs.x;
        let new_y = self.y + rhs.y;
        return Self::new(new_x, new_y);
    }
}
impl<T: Sub<Output = T> + Copy> Sub<Xy<T>> for Xy<T> {
    type Output = Self;
    // todo: track_caller everywhere else (or use someone else's math library)
    #[track_caller]
    fn sub(self, rhs: Xy<T>) -> Self::Output {
        let new_x = self.x - rhs.x;
        let new_y = self.y - rhs.y;
        return Self::new(new_x, new_y);
    }
}
impl<T: Add<Output = T> + Copy> Add<(T, T)> for Xy<T> {
    type Output = Self;
    fn add(self, rhs: (T, T)) -> Self::Output {
        let new_x = self.x + rhs.0;
        let new_y = self.y + rhs.1;
        return Self::new(new_x, new_y);
    }
}

impl<T> Index<Axis> for Xy<T> {
    type Output = T;
    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => return &self.x,
            Axis::Y => return &self.y,
        }
    }
}
impl<T> IndexMut<Axis> for Xy<T> {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => return &mut self.x,
            Axis::Y => return &mut self.y,
        }
    }
}
unsafe impl Zeroable for Xy<f32> {}
unsafe impl Pod for Xy<f32> {}
unsafe impl Zeroable for Xy<[f32; 2]> {}
unsafe impl Pod for Xy<[f32; 2]> {}

impl<T: Copy> Xy<T> {
    pub const fn new(x: T, y: T) -> Self {
        return Self { x, y };
    }

    pub const fn new_symm(v: T) -> Self {
        return Self { x: v, y: v };
    }
}

pub(crate) fn intersect(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let left = a[0].max(b[0]);
    let right = a[1].min(b[1]);
    return [left, right];
}

/// A two-dimensional rectangle.
/// 
/// Alias for [`Xy`]<[f32; 2]>.
/// 
/// ```rust
/// # use keru::*;
/// # let x0 = -1.0;
/// # let x1 =  1.0;
/// # let y0 = -1.0;
/// # let y1 =  1.0;
/// let rect = XyRect {
///     x: [x0, x1],
///     y: [y0, y1],
/// };
/// ``` 
pub type XyRect = Xy<[f32; 2]>;

impl XyRect {
    pub fn from_ends(start: Xy<f32>, end: Xy<f32>) -> XyRect {
        return XyRect {
            x: [start[X], end[Y]],
            y: [start[X], end[Y]],
        }
    }

    pub fn size(&self) -> Xy<f32> {
        return Xy::new(self[X][1] - self[X][0], self[Y][1] - self[Y][0]);
    }

    pub fn to_graphics_space_rounded(&self, size: Xy<f32>) -> Self {
        // Round to pixel grid
        let pixel_x = [
            (self.x[0] * size.x).round() / size.x,
            (self.x[1] * size.x).round() / size.x
        ];
        let pixel_y = [
            (self.y[0] * size.y).round() / size.y,
            (self.y[1] * size.y).round() / size.y
        ];
        
        let pixel_aligned = Self::new(pixel_x, pixel_y);
        
        // Then do the normal conversion
        let a = pixel_aligned * 2. - 1.;
        return Self::new([a.x[0], a.x[1]], [-a.y[1], -a.y[0]]);
    }

    pub fn start(&self) -> Xy<f32> {
        return Xy::new(self[X][0], self[Y][0]);
    }

    pub fn end(&self) -> Xy<f32> {
        return Xy::new(self[X][1], self[Y][1]);
    }
}
impl Add<f32> for XyRect {
    type Output = Self;
    fn add(self, rhs: f32) -> Self::Output {
        return Self::new(
            [self[X][0] + rhs, self[X][1] + rhs],
            [self[Y][0] + rhs, self[Y][1] + rhs],
        );
    }
}
impl Sub<f32> for XyRect {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self::Output {
        return Self::new(
            [self[X][0] - rhs, self[X][1] - rhs],
            [self[Y][0] - rhs, self[Y][1] - rhs],
        );
    }
}
impl Mul<f32> for XyRect {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        return Self::new(
            [self[X][0] * rhs, self[X][1] * rhs],
            [self[Y][0] * rhs, self[Y][1] * rhs],
        );
    }
}

impl Mul<Xy<f32>> for XyRect {
    type Output = Self;
    fn mul(self, rhs: Xy<f32>) -> Self::Output {
        return Self::new(
            [self[X][0] * rhs.x, self[X][1] * rhs.x],
            [self[Y][0] * rhs.y, self[Y][1] * rhs.y],
        );
    }
}

impl Mul<XyRect> for XyRect {
    type Output = Self;
    fn mul(self, rhs: XyRect) -> Self::Output {
        return Self::new(
            [self[X][0] * rhs[X][0], self[X][1] * rhs[X][1]],
            [self[Y][0] * rhs[Y][0], self[Y][1] * rhs[Y][1]],
        );
    }
}

impl Mul<Xy<f32>> for Xy<f32> {
    type Output = Self;
    fn mul(self, rhs: Xy<f32>) -> Self::Output {
        return Self::new(
            self[X] * rhs[X],
            self[Y] * rhs[Y],
        );
    }
}

impl Div<Xy<f32>> for Xy<f32> {
    type Output = Self;
    fn div(self, rhs: Xy<f32>) -> Self::Output {
        return Self::new(
            self[X] / rhs[X],
            self[Y] / rhs[Y],
        );
    }
}

impl From<Xy<f32>> for Vec2 {
    fn from(val: Xy<f32>) -> Self {
        return vec2(val.x, val.y);
    }
}
