#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::entity::*;
use crate::vec::{Rect, Vec2};
use fontdue::Font;
use image::imageops::flip_vertical_in_place;
use image::io::Reader as ImageReader;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug, Copy, Clone)]
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

pub enum ImageFormat {
    RGBA8888,
    R8,
}

impl<'de> Deserialize<'de> for ImageFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "RGBA8888" => ImageFormat::RGBA8888,
            "R8" => ImageFormat::R8,
            _ => panic!("Unknown image format"),
        })
    }
}

#[derive(Deserialize)]
pub struct SpriteAtlas {
    pub file_name: String,
    pub format: ImageFormat,
    pub size: [u32; 2],
    pub sprites: HashMap<String, Vec<Sprite>>,

    #[serde(skip)]
    pub image: Vec<u8>,
}

impl SpriteAtlas {
    pub fn from_image(
        meta_file_path: &str,
        image_file_path: &str,
    ) -> Self {
        let meta = fs::read_to_string(meta_file_path).unwrap();
        let mut atlas: Self = serde_json::from_str(&meta).unwrap();

        let mut image = ImageReader::open(image_file_path)
            .unwrap()
            .decode()
            .unwrap();
        flip_vertical_in_place(&mut image);

        atlas.image = image.as_bytes().to_vec();

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
    pub fn from_sprite_atlas(
        sprite_atlas: &SpriteAtlas,
        name: &'static str,
        duration: f32,
        scale: f32,
    ) -> Self {
        let frames = sprite_atlas.sprites.get(name).unwrap_or_else(|| {
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

    pub fn from_sprite(sprite: Sprite, flip: bool) -> Self {
        let size = Vec2::new(sprite.w, sprite.h).scale(sprite.scale);
        let rect = Rect::from_bot_center(Vec2::zeros(), size);

        Self {
            rect,
            color: None,
            sprite: Some(sprite),
            orientation: 0.0,
            flip,
        }
    }

    pub fn translate(&self, translation: Vec2<f32>) -> Self {
        let mut primitive = *self;
        primitive.rect = primitive.rect.translate(translation);

        primitive
    }
}

#[derive(Copy, Clone)]
pub struct Glyph {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub w_advance: f32,
    pub h_advance: f32,
}

pub struct GlyphAtlas {
    pub font: Font,
    pub size: [u32; 2],
    pub image: Vec<u8>,

    glyphs: Vec<Glyph>,
}

impl GlyphAtlas {
    pub fn from_ttf(file_path: &str, font_size: f32) -> Self {
        let font_bytes = fs::read(file_path).unwrap();
        let font =
            Font::from_bytes(font_bytes, fontdue::FontSettings::default())
                .unwrap();

        let mut metrics = Vec::new();
        let mut bitmaps = Vec::new();
        let mut image_width = 0;
        let mut image_height = 0;
        for u in 32..127 {
            let ch = char::from_u32(u).unwrap();
            let (metric, bitmap) = font.rasterize(ch, font_size);
            assert!(bitmap.len() == metric.width * metric.height);

            metrics.push(metric);
            bitmaps.push(bitmap);

            image_width = image_width.max(metric.width);
            image_height += metric.height;
        }

        let n_bytes = image_width * image_height;
        let mut image: Vec<u8> = vec![0; n_bytes];
        let mut glyphs = Vec::new();
        let mut cursor = 0;
        let mut y = 0.0;
        for i_glyph in 0..metrics.len() {
            let metric = &metrics[i_glyph];

            let name =
                char::from_u32(i_glyph as u32 + 32).unwrap().to_string();
            let glyph = Glyph {
                x: 0.0,
                y,
                w: metric.width as f32,
                h: metric.height as f32,
                w_advance: metric.advance_width,
                h_advance: metric.advance_height,
            };
            glyphs.push(glyph);

            let bitmap = &bitmaps[i_glyph];
            let n_bytes = bitmap.len();
            for i_glyph_row in 0..metric.height {
                let glyph_cursor = i_glyph_row * metric.width;
                let glyph_row =
                    &bitmap[glyph_cursor..glyph_cursor + metric.width];
                image[cursor..cursor + metric.width]
                    .copy_from_slice(glyph_row);
                cursor += image_width;
                y += 1.0;
            }
        }

        Self {
            font,
            size: [image_width as u32, image_height as u32],
            image,
            glyphs,
        }
    }

    pub fn get_glyph(&self, c: char) -> Glyph {
        let mut idx = c as usize;
        if idx < 32 || idx > 126 {
            idx = 63; // Question mark
        }

        self.glyphs[idx - 32]
    }
}

pub fn draw_entity(entity: &Entity, draw_queue: &mut Vec<DrawPrimitive>) {
    let position = entity.position;

    let animator = entity.animator.as_ref();
    let text = entity.text.as_ref();
    let health = entity.health.as_ref();

    let mut primitive;
    if let Some(animator) = animator {
        primitive = Some(animator.get_draw_primitive());
    } else {
        primitive = entity.draw_primitive;
    }

    if let Some(primitive) = primitive {
        draw_queue.push(primitive.translate(position));
    }

    if let (Some(primitive), Some(health)) = (primitive, health) {
        let alive_color = Color::new(0.0, 1.0, 0.0, 1.0);
        let dead_color = Color::new(1.0, 0.0, 0.0, 1.0);
        let ratio = health.current / health.max;
        let color = alive_color.lerp(&dead_color, ratio);
        let gap_height = 0.2;
        let bar_size = Vec2::new(1.0, 0.13);
        let border_size = Vec2::new(0.03, 0.03);

        let y = primitive.rect.get_top_left().y + gap_height;
        let bot_center = Vec2::new(0.0, y);
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

        draw_queue.push(background_primitive.translate(position));
        draw_queue.push(health_primitive.translate(position));
    }

    if let Some(text) = text {
        for primitive in text.draw_primitives.iter() {
            draw_queue.push(primitive.translate(position));
        }
    }
}
