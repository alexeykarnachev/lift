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

pub enum Origin {
    Center(Vec2<f32>),
    BotCenter(Vec2<f32>),
    BotLeft(Vec2<f32>),
    RightCenter(Vec2<f32>),
    LeftCenter(Vec2<f32>),
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub bot_left: Vec2<f32>,
    pub top_right: Vec2<f32>,
}

impl Rect {
    pub fn from_origin(origin: Origin, size: Vec2<f32>) -> Self {
        use Origin::*;
        return match origin {
            Center(p) => Self::from_center(p, size),
            BotCenter(p) => Self::from_bot_center(p, size),
            BotLeft(p) => Self::from_bot_left(p, size),
            RightCenter(p) => Self::from_right_center(p, size),
            LeftCenter(p) => Self::from_left_center(p, size),
        };
    }

    pub fn from_center(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let half_size = size.scale(0.5);

        Self {
            bot_left: position - half_size,
            top_right: position + half_size,
        }
    }

    pub fn from_bot_center(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let mut center = position;
        center.y += size.y * 0.5;

        Self::from_center(center, size)
    }

    pub fn from_bot_left(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let center = position + size.scale(0.5);

        Self::from_center(center, size)
    }

    pub fn from_right_center(
        position: Vec2<f32>,
        size: Vec2<f32>,
    ) -> Self {
        let mut center = position;
        center.x += size.x * 0.5;

        Self::from_center(center, size)
    }

    pub fn from_left_center(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let mut center = position;
        center.x -= size.x * 0.5;

        Self::from_center(center, size)
    }

    pub fn with_center(&self, position: Vec2<f32>) -> Self {
        Self::from_center(position, self.get_size())
    }

    pub fn with_bot_center(&self, position: Vec2<f32>) -> Self {
        Self::from_bot_center(position, self.get_size())
    }

    pub fn translate(&self, translation: Vec2<f32>) -> Self {
        Self {
            bot_left: self.bot_left + translation,
            top_right: self.top_right + translation,
        }
    }

    pub fn get_width(&self) -> f32 {
        self.get_size().x
    }

    pub fn get_center(&self) -> Vec2<f32> {
        (self.top_right + self.bot_left).scale(0.5)
    }

    pub fn get_bot_center(&self) -> Vec2<f32> {
        let mut center = self.get_center();
        center.y -= 0.5 * self.get_size().y;

        center
    }

    pub fn get_top_left(&self) -> Vec2<f32> {
        let mut top_left = self.top_right;
        top_left.x -= self.get_size().x;

        top_left
    }

    pub fn get_size(&self) -> Vec2<f32> {
        self.top_right - self.bot_left
    }

    pub fn get_x_dist_to(&self, x: f32) -> f32 {
        let left_dist = (x - self.bot_left.x).abs();
        let right_dist = (x - self.top_right.x).abs();

        left_dist.min(right_dist)
    }

    pub fn to_xywh(&self) -> [f32; 4] {
        let center = self.get_center();
        let size = self.get_size();

        [center.x, center.y, size.x, size.y]
    }
}
