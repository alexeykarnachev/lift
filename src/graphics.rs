#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::entity::*;
use crate::level::Level;
use crate::ui::*;
use crate::vec::{Origin, Rect, Vec2};
use core::fmt::Debug;
use enum_iterator::{all, Sequence};
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
    pub origin: Origin,
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
    pub sprite_duration: f32,

    #[serde(skip)]
    pub image: Vec<u8>,
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

        atlas.image = image.as_bytes().to_vec();

        atlas
    }
}

#[derive(Copy, Clone)]
pub struct Light {
    pub position: Vec2<f32>,
    pub color: Color,
    pub attenuation: [f32; 3],
}

#[derive(Copy, Clone, PartialEq)]
pub enum AnimationMode {
    Repeat,
    Once,
}

#[derive(Clone)]
pub struct AnimatedSprite {
    pub name: &'static str,
    pub duration: f32,
    pub animation_mode: AnimationMode,
    pub origin: Origin,
    pub cycle: f32,

    frames: Vec<Sprite>,
}

impl AnimatedSprite {
    pub fn new(
        sprite_atlas: &SpriteAtlas,
        name: &'static str,
        duration: f32,
        animation_mode: AnimationMode,
        origin: Origin,
    ) -> Self {
        let frames = sprite_atlas.sprites.get(name).unwrap_or_else(|| {
            panic!("There is no such sprite in the sprite atlas: {}", name)
        });

        Self {
            name,
            duration,
            animation_mode,
            origin,
            cycle: 0.0,
            frames: frames.to_vec(),
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 0.0;
    }

    pub fn is_finished(&self) -> bool {
        self.animation_mode == AnimationMode::Once && self.cycle == 1.0
    }

    pub fn update(&mut self, dt: f32) {
        use AnimationMode::*;

        self.cycle += dt / self.duration;

        match self.animation_mode {
            Once => {
                self.cycle = self.cycle.min(1.0);
            }
            Repeat => {
                self.cycle -= self.cycle.floor();
            }
        }

        assert!(self.cycle <= 1.0);
    }

    pub fn get_current_frame(&self) -> Sprite {
        let max_idx = (self.frames.len() - 1) as f32;
        let frame_idx = (self.cycle * max_idx).round() as usize;

        let mut frame = self.frames[frame_idx];
        frame.origin = self.origin;

        frame
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_slice(arr: &[f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    pub fn gray(c: f32, a: f32) -> Self {
        Self::new(c, c, c, a)
    }

    pub fn red(a: f32) -> Self {
        Self::new(1.0, 0.0, 0.0, a)
    }

    pub fn green(a: f32) -> Self {
        Self::new(0.0, 1.0, 0.0, a)
    }

    pub fn yellow(a: f32) -> Self {
        Self::new(1.0, 1.0, 0.0, a)
    }

    pub fn healthbar(ratio: f32) -> Self {
        let alive = Self::new(0.8, 0.0, 0.0, 1.0);
        let dead = Self::new(0.3, 0.0, 0.0, 1.0);

        alive.lerp(&dead, ratio)
    }

    pub fn staminabar(ratio: f32) -> Self {
        let alive = Self::new(0.0, 0.8, 0.0, 1.0);
        let dead = Self::new(0.0, 0.3, 0.0, 1.0);

        alive.lerp(&dead, ratio)
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

    pub fn to_rgb_array(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

#[derive(Copy, Clone)]
pub struct DrawPrimitive {
    pub rect: Rect,
    pub space: SpaceType,
    pub effect: u32,
    pub flip: bool,
    pub z: f32,

    pub color: Option<Color>,
    pub sprite: Option<Sprite>,
    pub tex: Option<TextureType>,
}

impl DrawPrimitive {
    pub fn from_rect(
        rect: Rect,
        space: SpaceType,
        z: f32,
        effect: u32,
        color: Color,
    ) -> Self {
        Self {
            rect,
            space,
            z,
            effect,
            flip: false,
            color: Some(color),
            sprite: None,
            tex: None,
        }
    }

    pub fn from_sprite(
        space: SpaceType,
        z: f32,
        effect: u32,
        position: Vec2<f32>,
        sprite: Sprite,
        color: Option<Color>,
        flip: bool,
        tex: TextureType,
    ) -> Self {
        let size = Vec2::new(sprite.w, sprite.h);
        let rect = Rect::from_origin(sprite.origin, position, size);

        Self {
            rect,
            space,
            z,
            effect,
            color,
            sprite: Some(sprite),
            tex: Some(tex),
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
    font_size_to_glyphs: HashMap<u32, Vec<Glyph>>,
}

impl GlyphAtlas {
    pub fn new(file_path: &str, font_sizes: &[u32]) -> Self {
        let mut font_size_to_glyphs = HashMap::<u32, Vec<Glyph>>::new();
        for font_size in font_sizes {
            font_size_to_glyphs.insert(*font_size, Vec::new());
        }

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

            for font_size in font_sizes {
                let (metric, bitmap) =
                    font.rasterize(ch, *font_size as f32);

                assert!(bitmap.len() == metric.width * metric.height);

                metrics.push(metric);
                bitmaps.push(bitmap);

                max_glyph_width = max_glyph_width.max(metric.width);
                max_glyph_height = max_glyph_height.max(metric.height);
            }
        }

        let n_glyphs = metrics.len();
        let n_bytes_per_glyph = max_glyph_width * max_glyph_height;
        let n_glyphs_per_row = (n_glyphs as f32).sqrt().ceil() as usize;
        let image_height = max_glyph_height * n_glyphs_per_row;
        let image_width = max_glyph_width * n_glyphs_per_row;
        let mut image = vec![0u8; image_width * image_height];
        for i_glyph in 0..n_glyphs {
            let ir = (i_glyph / n_glyphs_per_row) * max_glyph_height;
            let ic = (i_glyph % n_glyphs_per_row) * max_glyph_width;
            let metric = &metrics[i_glyph];
            let glyph = Glyph {
                x: ic as f32,
                y: (image_height - ir) as f32,
                metrics: *metric,
            };
            let font_size_idx = i_glyph % font_sizes.len();
            let font_size = font_sizes[font_size_idx];
            font_size_to_glyphs.get_mut(&font_size).unwrap().push(glyph);

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
            font_size_to_glyphs,
        }
    }

    pub fn get_glyph(&self, c: char, font_size: u32) -> Glyph {
        let glyphs = self.font_size_to_glyphs.get(&font_size).unwrap();

        let mut idx = c as usize;
        if idx < 32 || idx > 126 {
            idx = 63; // Question mark
        }

        glyphs[idx - 32]
    }
}

pub fn draw_entity(entity: &Entity, draw_queue: &mut Vec<DrawPrimitive>) {
    if let Some(animator) = entity.animator.as_ref() {
        let mut primitive =
            animator.get_draw_primitive(entity.position, entity.effect);

        let t = entity.get_time_since_last_received_damage();
        if t < 0.1 {
            primitive.color = Some(Color::red(0.5));
        }

        draw_queue.push(primitive);
    } else if let Some(collider) = entity.get_collider() {
        let primitive = DrawPrimitive::from_rect(
            collider,
            SpaceType::WorldSpace,
            0.0,
            0,
            Color::new(1.0, 0.0, 0.0, 0.75),
        );
        draw_queue.push(primitive);
    };

    if entity.check_if_dead() {
        return;
    }

    draw_healthbar(entity, draw_queue);
}

fn draw_healthbar(entity: &Entity, draw_queue: &mut Vec<DrawPrimitive>) {
    let fade_in_time = 0.0;
    let plateau_time = 3.0;
    let fade_out_time = 0.3;

    let t = entity.get_time_since_last_received_damage();
    let a = if t <= fade_in_time {
        t / fade_in_time
    } else if t <= (fade_in_time + plateau_time) {
        1.0
    } else {
        let total_time = fade_in_time + plateau_time + fade_out_time;
        ((total_time - t) / fade_out_time).max(0.0)
    };

    let alive_color = Color::new(0.0, 1.0, 0.0, a);
    let dead_color = Color::new(1.0, 0.0, 0.0, a);
    let ratio = entity.get_health_ratio();
    let color = alive_color.lerp(&dead_color, ratio);
    let bar_size = Vec2::new(20.0, 2.6);
    let border_size = Vec2::new(0.6, 0.6);

    let y = entity.get_top_left().y + 4.0;
    let position = Vec2::new(entity.get_center().x, y);
    let background_rect = Rect::from_bot_center(position, bar_size);
    draw_queue.push(DrawPrimitive::from_rect(
        background_rect,
        SpaceType::WorldSpace,
        0.0,
        0,
        Color::gray(0.2, a),
    ));

    let bot_left = background_rect.bot_left + border_size;
    let mut bar_size = bar_size - border_size.scale(2.0);
    bar_size.x *= ratio;
    let health_rect = Rect::from_bot_left(bot_left, bar_size);
    draw_queue.push(DrawPrimitive::from_rect(
        health_rect,
        SpaceType::WorldSpace,
        0.0,
        0,
        color,
    ));
}

pub fn draw_bullet(bullet: &Bullet, draw_queue: &mut Vec<DrawPrimitive>) {
    let rect = bullet.get_collider();
    draw_queue.push(DrawPrimitive::from_rect(
        rect,
        SpaceType::WorldSpace,
        0.0,
        0,
        Color::red(1.0),
    ));
}

pub fn draw_collider(collider: Rect, draw_queue: &mut Vec<DrawPrimitive>) {
    // draw_queue.push(DrawPrimitive::from_rect(
    //     collider,
    //     SpaceType::WorldSpace,
    //     0.0,
    //     0,
    //     Color::new(1.0, 0.0, 0.0, 0.5),
    // ));
}

pub fn draw_attack(attack: &Attack, draw_queue: &mut Vec<DrawPrimitive>) {
    // let rect = attack.get_collider();
    // draw_queue.push(DrawPrimitive::from_rect(
    //     rect,
    //     SpaceType::WorldSpace,
    //     0.0,
    //     0,
    //     Color::yellow(0.5),
    // ));
}

pub fn draw_level(level: &Level, draw_queue: &mut Vec<DrawPrimitive>) {
    draw_queue.extend_from_slice(&level.draw_primitives);
}

pub fn draw_text(text: &Text, draw_queue: &mut Vec<DrawPrimitive>) {
    draw_queue.extend_from_slice(&text.get_draw_primitives());
}

pub fn draw_ui(ui: &UI, draw_queue: &mut Vec<DrawPrimitive>) {
    ui.texts.iter().for_each(|t| draw_text(t, draw_queue));
    ui.rects.iter().for_each(|r| draw_queue.push(*r));
}

#[derive(Copy, Clone, Debug, Sequence)]
pub enum TextureType {
    SpriteTexture = 1,
    GlyphTexture = 2,
}

impl From<TextureType> for u32 {
    fn from(e: TextureType) -> u32 {
        e as u32
    }
}

#[derive(Copy, Clone, Debug, Sequence)]
pub enum SpaceType {
    WorldSpace = 1,
    CameraSpace = 2,
    ScreenSpace = 3,
}

impl From<SpaceType> for u32 {
    fn from(e: SpaceType) -> u32 {
        e as u32
    }
}

#[derive(Copy, Clone, Debug, Sequence)]
pub enum EffectType {
    ApplyLightEffect = 1 << 0,
    StoneWallEffect = 1 << 1,
}

impl From<EffectType> for u32 {
    fn from(e: EffectType) -> u32 {
        e as u32
    }
}

pub fn enum_to_shader_source<T: Sequence + Debug + Copy + Into<u32>>(
) -> String {
    let mut source = String::new();

    for variant in all::<T>().collect::<Vec<_>>() {
        let definition =
            format!("const uint {:?} = {:?};\n", variant, variant.into());
        source.push_str(&definition);
    }

    source
}
