#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::entity::*;
use crate::vec::{Rect, Vec2};
use image::imageops::flip_vertical_in_place;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Copy, Clone)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,

    #[serde(skip)]
    pub scale: f32,
}

impl Sprite {
    pub fn to_tex_xywh(&self) -> [f32; 4] {
        [self.x, self.y, self.w, self.h]
    }
}

#[derive(Deserialize)]
pub struct SpriteAtlas {
    pub file_name: String,
    pub size: [u32; 2],
    pub sprites: HashMap<String, Vec<Sprite>>,

    #[serde(skip)]
    pub image: DynamicImage,
}

impl SpriteAtlas {
    pub fn new(meta_file_path: &str, image_file_path: &str) -> Self {
        let meta = fs::read_to_string(meta_file_path).unwrap();
        let mut atlas: Self = serde_json::from_str(&meta).unwrap();

        let mut image = ImageReader::open(image_file_path)
            .unwrap()
            .decode()
            .unwrap();
        flip_vertical_in_place(&mut image);

        atlas.image = image;

        atlas
    }
}

pub struct AnimatedSprite {
    pub name: &'static str,
    pub duration: f32,
    pub scale: f32,
    current_duration: f32,

    frames: Vec<Sprite>,
}

impl AnimatedSprite {
    pub fn from_atlas(
        atlas: &SpriteAtlas,
        name: &'static str,
        duration: f32,
        scale: f32,
    ) -> Self {
        let frames = atlas.sprites.get(name).unwrap_or_else(|| {
            panic!("There is no such sprite in the sprite atlas: {}", name)
        });

        Self {
            name,
            duration,
            scale,
            current_duration: 0.0,
            frames: frames.to_vec(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.current_duration += dt;
    }

    pub fn get_current_frame(&self) -> Sprite {
        let mut cycle = self.current_duration / self.duration;
        cycle -= cycle.floor();
        let frame_idx = (cycle * self.frames.len() as f32).floor();

        let mut frame = self.frames[frame_idx as usize];
        frame.scale = self.scale;

        frame
    }
}

#[derive(Copy, Clone)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn new_gray(c: f32, a: f32) -> Self {
        Self {
            r: c,
            g: c,
            b: c,
            a: a,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn lerp(&self, other: &Self, k: f32) -> Self {
        let k_other = 1.0 - k;
        Self {
            r: k * self.r + k_other * other.r,
            g: k * self.g + k_other * other.g,
            b: k * self.b + k_other * other.b,
            a: k * self.a + k_other * other.a,
        }
    }
}

#[derive(Copy, Clone)]
pub struct DrawPrimitive {
    pub rect: Rect,
    pub color: Option<Color>,
    pub sprite: Option<Sprite>,
    pub orientation: f32,
    pub flip: bool,
}

impl DrawPrimitive {
    pub fn with_color(rect: Rect, color: Color, orientation: f32) -> Self {
        Self {
            rect,
            color: Some(color),
            sprite: None,
            orientation,
            flip: false,
        }
    }

    pub fn from_sprite(
        sprite: Sprite,
        position: Vec2<f32>,
        flip: bool,
    ) -> Self {
        let size = Vec2::new(sprite.w, sprite.h).scale(sprite.scale);
        let rect = Rect::from_bot_center(position, size);

        Self {
            rect,
            color: None,
            sprite: Some(sprite),
            orientation: 0.0,
            flip,
        }
    }
}

pub fn draw_entity(entity: &Entity, draw_queue: &mut Vec<DrawPrimitive>) {
    let position = entity.position;

    let animator = entity.animator.as_ref();
    let health = entity.health.as_ref();

    let mut primitive = None;
    if let Some(animator) = animator {
        primitive = Some(animator.get_draw_primitive(position));
    } else if let Some(mut _primitive) = entity.draw_primitive {
        _primitive.rect = _primitive.rect.with_bot_center(position);
        primitive = Some(_primitive);
    }

    if let Some(primitive) = primitive {
        draw_queue.push(primitive);
    }

    if let (Some(primitive), Some(health)) = (primitive, health) {
        let alive_color = Color::new(0.0, 1.0, 0.0, 1.0);
        let dead_color = Color::new(1.0, 0.0, 0.0, 1.0);
        let ratio = health.current / health.max;
        let color = alive_color.lerp(&dead_color, ratio);
        let gap_height = 0.2;
        let bar_size = Vec2::new(1.0, 0.13);
        let border_size = Vec2::new(0.03, 0.03);

        let x = position.x;
        let y = primitive.rect.get_top_left().y + gap_height;
        let bot_center = Vec2::new(x, y);
        let background_rect = Rect::from_bot_center(bot_center, bar_size);
        let background_primitive = DrawPrimitive::with_color(
            background_rect,
            Color::new_gray(0.2, 1.0),
            0.0,
        );

        let bot_left = background_rect.bot_left + border_size;
        let mut bar_size = bar_size - border_size.scale(2.0);
        bar_size.x *= ratio;
        let health_rect = Rect::from_bot_left(bot_left, bar_size);
        let health_primitive =
            DrawPrimitive::with_color(health_rect, color, 0.0);

        draw_queue.push(background_primitive);
        draw_queue.push(health_primitive);
    }
}
