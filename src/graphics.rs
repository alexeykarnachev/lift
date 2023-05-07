#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::entity::*;
use crate::vec::{Origin, Rect, Vec2};
use fontdue::Font;
use fontdue::Metrics;
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
pub enum Texture {
    Sprite = 1,
    Glyph = 2,
}

#[derive(Copy, Clone)]
pub struct DrawPrimitive {
    pub rect: Rect,
    pub color: Option<Color>,
    pub sprite: Option<Sprite>,
    pub tex: Option<Texture>,
    pub orientation: f32,
    pub flip: bool,
}

impl DrawPrimitive {
    pub fn from_rect(rect: Rect, color: Color, orientation: f32) -> Self {
        Self {
            rect,
            color: Some(color),
            sprite: None,
            tex: None,
            orientation,
            flip: false,
        }
    }

    pub fn from_sprite(
        origin: Origin,
        sprite: Sprite,
        color: Option<Color>,
        flip: bool,
        tex: Texture,
    ) -> Self {
        let size = Vec2::new(sprite.w, sprite.h).scale(sprite.scale);
        let rect = Rect::from_origin(origin, size);

        Self {
            rect,
            color,
            sprite: Some(sprite),
            tex: Some(tex),
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
    pub metrics: Metrics,
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
        let mut max_glyph_width = 0;
        let mut max_glyph_height = 0;
        for u in 32..127 {
            let ch = char::from_u32(u).unwrap();
            let (metric, bitmap) = font.rasterize(ch, font_size);

            assert!(bitmap.len() == metric.width * metric.height);

            metrics.push(metric);
            bitmaps.push(bitmap);

            max_glyph_width = max_glyph_width.max(metric.width);
            max_glyph_height = max_glyph_height.max(metric.height);
        }

        let n_glyphs = metrics.len();
        let n_bytes_per_glyph = max_glyph_width * max_glyph_height;
        let n_glyphs_per_row = (n_glyphs as f32).sqrt().ceil() as usize;
        let image_height = max_glyph_height * n_glyphs_per_row;
        let image_width = max_glyph_width * n_glyphs_per_row;
        let mut image = vec![0u8; image_width * image_height];
        let mut glyphs = Vec::new();
        for i_glyph in 0..n_glyphs {
            let ir = (i_glyph / n_glyphs_per_row) * max_glyph_height;
            let ic = (i_glyph % n_glyphs_per_row) * max_glyph_width;

            let metric = &metrics[i_glyph];
            let glyph = Glyph {
                x: ic as f32,
                y: (image_height - ir - 1) as f32,
                metrics: *metric,
            };
            glyphs.push(glyph);
            let bitmap = &bitmaps[i_glyph];
            assert!(bitmap.len() == metric.width * metric.height);

            for gr in 0..metric.height {
                let start = gr * metric.width;
                let end = start + metric.width;
                let glyph_row = &bitmap[start..end];

                let start = (ir + gr) * image_width + ic;
                let end = start + metric.width;
                image[start..end].copy_from_slice(&glyph_row);
            }
        }

        let mut flipped_image = vec![0u8; image_width * image_height];
        for r in 0..image_height {
            let start = (image_height - r - 1) * image_width;
            let end = start + image_width;
            let source = &image[start..end];

            let start = r * image_width;
            let end = start + image_width;
            flipped_image[start..end].copy_from_slice(source);
        }

        Self {
            font,
            size: [image_width as u32, image_height as u32],
            image: flipped_image,
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
        let gap_height = 0.2;
        let y = primitive.rect.get_top_left().y + gap_height;
        for p in health.get_draw_primitives(Vec2::new(0.0, y)) {
            draw_queue.push(p.translate(position));
        }
    }

    if let Some(text) = text {
        for primitive in text.draw_primitives.iter() {
            draw_queue.push(primitive.translate(position));
        }
    }
}
