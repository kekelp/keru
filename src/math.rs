use bytemuck::{Pod, Zeroable};
use std::{hash::{Hash, Hasher}, ops::{Add, Div, Index, IndexMut, Mul, Sub}};
use Axis::*;

use crate::Ui;

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
    pub fn to_pixels(&self, len: Len, axis: Axis) -> u32 {
        match len {
            Len::Pixels(pixels) => return pixels,
            Len::Frac(frac) => return (frac * self.sys.part.unifs.size[axis]) as u32,
        }
    }

    pub fn f32_size_to_pixels2(&self, size: Xy<f32>) -> Xy<u32> {
        return Xy::new(
            (size.x * self.sys.part.unifs.size[X]) as u32,
            (size.y * self.sys.part.unifs.size[Y]) as u32
        );
    }

    pub fn to_pixels2(&self, len: Xy<Len>) -> Xy<u32> {
        return Xy::new(self.to_pixels(len.x, X), self.to_pixels(len.y, Y));
    }

    pub fn to_frac(&self, len: Len, axis: Axis) -> f32 {
        match len {
            Len::Pixels(pixels) => return (pixels as f32) / self.sys.part.unifs.size[axis],
            Len::Frac(frac) => return frac,
        }
    }

    pub fn pixels_to_frac(&self, pixels: u32, axis: Axis) -> f32 {
        return (pixels as f32) / self.sys.part.unifs.size[axis];
    }
    pub fn f32_pixels_to_frac(&self, pixels: f32, axis: Axis) -> f32 {
        return pixels / self.sys.part.unifs.size[axis];
    }

    pub fn f32_pixels_to_frac2(&self, pixels: Xy<f32>) -> Xy<f32> {
        return Xy::new(
            self.f32_pixels_to_frac(pixels.x, X),
            self.f32_pixels_to_frac(pixels.y, Y),
        );
    }

    pub fn to_frac2(&self, len: Xy<Len>) -> Xy<f32> {
        return Xy::new(self.to_frac(len.x, X), self.to_frac(len.y, Y));
    }
}

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

pub type XyRect = Xy<[f32; 2]>;

impl XyRect {
    pub fn size(&self) -> Xy<f32> {
        return Xy::new(self[X][1] - self[X][0], self[Y][1] - self[Y][0]);
    }

    pub fn rightward(origin: Xy<f32>, size: Xy<f32>) -> Self {
        return Self {
            x: [origin.x, origin.x + size.x],
            y: [origin.y, origin.y + size.y],
        };
    }

    pub fn leftward(origin: Xy<f32>, size: Xy<f32>) -> Self {
        return Self {
            x: [origin.x - size.x, origin.x],
            y: [origin.y - size.y, origin.y],
        };
    }

    pub fn from_center(origin: Xy<f32>, size: Xy<f32>) -> Self {
        return Self {
            x: [origin.x - size.x / 2.0, origin.x + size.x / 2.0],
            y: [origin.y - size.y / 2.0, origin.y + size.y / 2.0],
        };
    }

    pub fn to_graphics_space(self) -> Self {
        let a = self * 2. - 1.;
        return Self::new([a.x[0], a.x[1]], [-a.y[1], -a.y[0]]);
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