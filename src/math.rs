use std::ops::{Add, Index, IndexMut, Mul, Sub};
use Axis::*;
use bytemuck::{Pod, Zeroable};


#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Xy<T> {
    pub x: T,
    pub y: T,
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
            y: [origin.y, origin.y + size.y]
        }
    }

    pub fn leftward(origin: Xy<f32>, size: Xy<f32>) -> Self {
        return Self {
            x: [origin.x - size.x, origin.x],
            y: [origin.y - size.y, origin.y]
        }
    }

    pub fn from_center(origin: Xy<f32>, size: Xy<f32>) -> Self {
        return Self {
            x: [origin.x - size.x / 2.0, origin.x + size.x / 2.0],
            y: [origin.y - size.y / 2.0, origin.y + size.y / 2.0]
        }
    }

    pub fn to_graphics_space(self) -> Self {
        let a = self * 2. - 1.;
        return Self::new(
            [a.x[0], a.x[1]],
            [-a.y[1], -a.y[0]],
        );
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