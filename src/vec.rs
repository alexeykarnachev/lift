use num_traits::Float;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};

#[derive(Debug, Clone, Copy)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn to_array(&self) -> [T; 2] {
        [self.x, self.y]
    }

    pub fn with_y(&self, y: T) -> Self {
        Self { x: self.x, y }
    }

    pub fn with_x(&self, x: T) -> Self {
        Self { x, y: self.y }
    }
}

impl<T: Copy + Add<Output = T>> Vec2<T> {
    pub fn add_y(self, y: T) -> Self {
        Self {
            x: self.x,
            y: self.y + y,
        }
    }

    pub fn add_x(self, x: T) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
        }
    }
}

impl Vec2<f32> {
    pub fn from_orientation(orientation: f32) -> Self {
        Self {
            x: orientation.cos(),
            y: orientation.sin(),
        }
    }

    pub fn to_orientation(&self) -> f32 {
        self.y.atan2(self.x)
    }
}

impl<T: From<f32>> Vec2<T> {
    pub fn zeros() -> Self {
        Self {
            x: 0.0.into(),
            y: 0.0.into(),
        }
    }
}

impl<T: Float> Vec2<T> {
    pub fn abs(self) -> Vec2<T> {
        Vec2::new(self.x.abs(), self.y.abs())
    }

    pub fn abs_inplace(&mut self) {
        self.x = self.x.abs();
        self.y = self.y.abs();
    }
}

impl<T: Into<f32> + From<f32> + Copy> Vec2<T> {
    pub fn rotate(self, origin: Vec2<T>, angle: f32) -> Vec2<f32> {
        let cos: f32 = angle.cos().into();
        let sin: f32 = angle.sin().into();
        let x_diff = self.x.into() - origin.x.into();
        let y_diff = self.y.into() - origin.y.into();
        let rotated_x = origin.x.into() + (cos * x_diff) - (sin * y_diff);
        let rotated_y = origin.y.into() + (sin * x_diff) + (cos * y_diff);
        Vec2::<f32>::new(rotated_x, rotated_y)
    }

    pub fn rotate_inplace(&mut self, origin: Vec2<T>, angle: f32) {
        let rotated = self.rotate(origin, angle);
        self.x = rotated.x.into();
        self.y = rotated.y.into();
    }
}

impl<T: Add<Output = T>> Add for Vec2<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Copy + Add<Output = T>> AddAssign for Vec2<T> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl<T: Sub<Output = T>> Sub for Vec2<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Copy + Sub<Output = T>> SubAssign for Vec2<T> {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

impl<T: Mul<Output = T>> Mul for Vec2<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: Copy + Mul<Output = T>> MulAssign for Vec2<T> {
    fn mul_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x * other.x,
            y: self.y * other.y,
        };
    }
}

impl<T: Div<Output = T>> Div for Vec2<T> {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl<T: Copy + Div<Output = T>> DivAssign for Vec2<T> {
    fn div_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x / other.x,
            y: self.y / other.y,
        };
    }
}

impl<T: Float> Vec2<T> {
    pub fn len(self) -> T {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn scale(self, k: T) -> Vec2<T> {
        Vec2::new(self.x * k, self.y * k)
    }

    pub fn norm(self) -> Vec2<T> {
        self.scale(self.len().recip())
    }

    pub fn dist_to(self, other: Self) -> T {
        (other - self).len()
    }

    pub fn with_len(self, len: T) -> Vec2<T> {
        self.norm().scale(len)
    }
}
