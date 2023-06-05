use crate::utils::frand;
use num_traits::Float;
use std::f32::consts::PI;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};

#[derive(Debug, Clone, Copy, Default)]
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

impl<T: Copy + Mul<Output = T>> Vec2<T> {
    pub fn mul_y(self, y: T) -> Self {
        Self {
            x: self.x,
            y: self.y * y,
        }
    }

    pub fn mul_x(self, x: T) -> Self {
        Self {
            x: self.x * x,
            y: self.y,
        }
    }
}

impl<T: From<f32>> Vec2<T> {
    pub fn zeros() -> Self {
        Self {
            x: 0.0.into(),
            y: 0.0.into(),
        }
    }

    pub fn right() -> Self {
        Self {
            x: 1.0.into(),
            y: 0.0.into(),
        }
    }

    pub fn left() -> Self {
        Self {
            x: (-1.0).into(),
            y: 0.0.into(),
        }
    }

    pub fn up() -> Self {
        Self {
            x: 0.0.into(),
            y: 1.0.into(),
        }
    }

    pub fn down() -> Self {
        Self {
            x: 0.0.into(),
            y: (-1.0).into(),
        }
    }

    pub fn from_angle(theta: f32) -> Self {
        Self {
            x: theta.cos().into(),
            y: theta.sin().into(),
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

impl Vec2<f32> {
    pub fn frand(range: (f32, f32)) -> Vec2<f32> {
        Vec2::new(frand(-range.0, range.0), frand(-range.1, range.1))
    }

    pub fn rnd_on_circle(radius: f32) -> Vec2<f32> {
        let theta = frand(0.0, 2.0 * PI);

        Vec2::from_angle(theta).scale(radius)
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

#[derive(Clone, Copy, Debug)]
pub enum Pivot {
    Center(Vec2<f32>),
    BotCenter(Vec2<f32>),
    TopCenter(Vec2<f32>),
    BotLeft(Vec2<f32>),
    TopLeft(Vec2<f32>),
    TopRight(Vec2<f32>),
    RightCenter(Vec2<f32>),
    LeftCenter(Vec2<f32>),
}

impl Default for Pivot {
    fn default() -> Self {
        Self::Center(Vec2::zeros())
    }
}

impl Pivot {
    pub fn from_str(name: &str, position: Vec2<f32>) -> Self {
        match name {
            "Center" => Self::Center(position),
            "BotCenter" => Self::BotCenter(position),
            "TopCenter" => Self::TopCenter(position),
            "BotLeft" => Self::BotLeft(position),
            "TopLeft" => Self::TopLeft(position),
            "TopRight" => Self::TopRight(position),
            "RightCenter" => Self::RightCenter(position),
            "LeftCenter" => Self::LeftCenter(position),
            _ => {
                panic!("Unknown Pivot: {}", name)
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Rect {
    bot_left: Vec2<f32>,
    top_right: Vec2<f32>,
}

impl Rect {
    pub fn zeros() -> Self {
        Self {
            bot_left: Vec2::zeros(),
            top_right: Vec2::zeros(),
        }
    }

    pub fn from_pivot(pivot: Pivot, size: Vec2<f32>) -> Self {
        use Pivot::*;
        return match pivot {
            Center(p) => Self::from_center(p, size),
            BotCenter(p) => Self::from_bot_center(p, size),
            TopCenter(p) => Self::from_top_center(p, size),
            BotLeft(p) => Self::from_bot_left(p, size),
            TopLeft(p) => Self::from_top_left(p, size),
            TopRight(p) => Self::from_top_right(p, size),
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

    pub fn from_top_center(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let mut center = position;
        center.y -= size.y * 0.5;

        Self::from_center(center, size)
    }

    pub fn from_bot_left(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let center = position + size.scale(0.5);

        Self::from_center(center, size)
    }

    pub fn from_bot_right(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let center = position + Vec2::new(-size.x * 0.5, size.y * 0.5);

        Self::from_center(center, size)
    }

    pub fn from_top_left(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let center = position + Vec2::new(size.x * 0.5, -size.y * 0.5);

        Self::from_center(center, size)
    }

    pub fn from_top_right(position: Vec2<f32>, size: Vec2<f32>) -> Self {
        let center = position - size.scale(0.5);

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
        center.x += size.x * 0.5;

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

    pub fn get_height(&self) -> f32 {
        self.get_size().y
    }

    pub fn get_bot_left(&self) -> Vec2<f32> {
        self.bot_left
    }

    pub fn get_top_right(&self) -> Vec2<f32> {
        self.top_right
    }

    pub fn get_center(&self) -> Vec2<f32> {
        (self.top_right + self.bot_left).scale(0.5)
    }

    pub fn get_bot_center(&self) -> Vec2<f32> {
        let mut center = self.get_center();
        center.y -= 0.5 * self.get_size().y;

        center
    }

    pub fn get_top_center(&self) -> Vec2<f32> {
        let mut center = self.get_center();
        center.y += 0.5 * self.get_size().y;

        center
    }

    pub fn get_top_left(&self) -> Vec2<f32> {
        let mut top_left = self.top_right;
        top_left.x -= self.get_size().x;

        top_left
    }

    pub fn get_bot_right(&self) -> Vec2<f32> {
        let mut bot_right = self.top_right;
        bot_right.y -= self.get_size().y;

        bot_right
    }

    pub fn get_right_center(&self) -> Vec2<f32> {
        let mut center = self.top_right;
        center.y -= 0.5 * self.get_size().y;

        center
    }

    pub fn get_size(&self) -> Vec2<f32> {
        self.top_right - self.bot_left
    }

    pub fn get_x_dist_to(&self, x: f32) -> f32 {
        let left_dist = (x - self.bot_left.x).abs();
        let right_dist = (x - self.top_right.x).abs();

        left_dist.min(right_dist)
    }

    pub fn get_y_min(&self) -> f32 {
        self.bot_left.y
    }

    pub fn get_y_max(&self) -> f32 {
        self.top_right.y
    }

    pub fn get_x_min(&self) -> f32 {
        self.bot_left.x
    }

    pub fn get_x_max(&self) -> f32 {
        self.top_right.x
    }

    pub fn merge(&self, other: Rect) -> Self {
        let bot_left = Vec2::new(
            self.bot_left.x.min(other.bot_left.x),
            self.bot_left.y.min(other.bot_left.y),
        );
        let top_right = Vec2::new(
            self.top_right.x.max(other.top_right.x),
            self.top_right.y.max(other.top_right.y),
        );

        Self {
            bot_left,
            top_right,
        }
    }

    pub fn expand_from_center(
        &self,
        add_width: f32,
        add_height: f32,
    ) -> Self {
        let center = self.get_center();
        let mut size = self.get_size();
        size.x += add_width;
        size.y += add_height;

        Self::from_center(center, size)
    }

    pub fn check_if_collides_with_point(&self, p: Vec2<f32>) -> bool {
        p.x > self.bot_left.x
            && p.x < self.top_right.x
            && p.y > self.bot_left.y
            && p.y < self.top_right.y
    }

    pub fn check_if_collides_with_rect(&self, rect: Rect) -> bool {
        let sum_width = self.get_width() + rect.get_width();
        let sum_height = self.get_height() + rect.get_height();

        let min_x = self.bot_left.x.min(rect.bot_left.x);
        let max_x = self.top_right.x.max(rect.top_right.x);
        let min_y = self.bot_left.y.min(rect.bot_left.y);
        let max_y = self.top_right.y.max(rect.top_right.y);

        let width = max_x - min_x;
        let height = max_y - min_y;

        width <= sum_width && height <= sum_height
    }

    pub fn check_if_collides_with_line(
        &self,
        start: Vec2<f32>,
        end: Vec2<f32>,
    ) -> bool {
        let mut x_min = start.x.min(end.x);
        let mut x_max = start.x.max(end.x);
        let rect_x_min = self.get_x_min();
        let rect_x_max = self.get_x_max();
        let rect_y_min = self.get_y_min();
        let rect_y_max = self.get_y_max();

        x_max = x_max.min(rect_x_max);
        x_min = x_min.max(rect_x_min);

        if x_min > x_max {
            return false;
        }

        let dx = end.x - start.x;

        let mut y_min = start.y;
        let mut y_max = end.y;
        if dx.abs() > f32::EPSILON {
            let a = (end.y - start.y) / dx;
            let b = start.y - a * start.x;
            y_min = a * x_min + b;
            y_max = a * x_max + b;
        }

        if y_min > y_max {
            let tmp = y_max;
            y_max = y_min;
            y_min = tmp;
        }

        y_max = y_max.min(rect_y_max);
        y_min = y_min.max(rect_y_min);

        if y_min > y_max {
            return false;
        }

        true
    }

    pub fn collide_aabb(&self, other: Rect) -> Vec2<f32> {
        let mut mtv = Vec2::zeros();

        if self.check_if_collides_with_rect(other) {
            mtv.x = if other.get_x_max() > self.get_x_max() {
                other.get_x_min() - self.get_x_max()
            } else {
                other.get_x_max() - self.get_x_min()
            };

            mtv.y = if other.get_y_max() > self.get_y_max() {
                other.get_y_min() - self.get_y_max()
            } else {
                other.get_y_max() - self.get_y_min()
            };

            if mtv.x.abs() > mtv.y.abs() {
                mtv.x = 0.0;
            } else {
                mtv.y = 0.0;
            }
        }

        mtv
    }

    pub fn to_xywh(&self) -> [f32; 4] {
        let center = self.get_center();
        let size = self.get_size();

        [center.x, center.y, size.x, size.y]
    }
}
