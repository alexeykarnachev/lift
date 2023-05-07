#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::graphics::*;
use crate::vec::Vec2;
use crate::vec::*;
use std::collections::HashMap;
use std::fs;

pub struct Entity {
    pub position: Vec2<f32>,
    pub collider: Option<Rect>,
    pub kinematic: Option<Kinematic>,
    pub health: Option<Health>,
    pub weapon: Option<Weapon>,
    pub draw_primitive: Option<DrawPrimitive>,
    pub animator: Option<Animator>,
    pub text: Option<Text>,
}

impl Entity {
    pub fn new(position: Vec2<f32>) -> Self {
        Self {
            position,
            collider: None,
            kinematic: None,
            health: None,
            weapon: None,
            draw_primitive: None,
            animator: None,
            text: None,
        }
    }

    pub fn get_collider(&self) -> Rect {
        self.collider
            .as_ref()
            .unwrap()
            .with_bot_center(self.position)
    }

    pub fn get_draw_primitive_size(&self) -> Vec2<f32> {
        self.draw_primitive.as_ref().unwrap().rect.get_size()
    }

    pub fn set_draw_primitive_color(&mut self, color: Color) {
        self.draw_primitive.as_mut().unwrap().color = Some(color);
    }

    pub fn update_animator(&mut self, dt: f32) {
        self.animator.as_mut().unwrap().update(dt);
    }

    pub fn is_dead(&self) -> bool {
        self.health.as_ref().unwrap().current <= 0.0
    }
}

pub struct Kinematic {
    pub max_speed: f32,
    pub speed: f32,
    pub target: Option<Vec2<f32>>,
}

pub struct Health {
    pub max: f32,
    pub current: f32,
}

impl Health {
    pub fn get_draw_primitives(
        &self,
        position: Vec2<f32>,
    ) -> [DrawPrimitive; 2] {
        let alive_color = Color::new(0.0, 1.0, 0.0, 1.0);
        let dead_color = Color::new(1.0, 0.0, 0.0, 1.0);
        let ratio = self.current / self.max;
        let color = alive_color.lerp(&dead_color, ratio);
        let bar_size = Vec2::new(1.0, 0.13);
        let border_size = Vec2::new(0.03, 0.03);

        let background_rect = Rect::from_bot_center(position, bar_size);
        let background_primitive = DrawPrimitive::from_rect(
            background_rect,
            Color::new_gray(0.2, 1.0),
            0.0,
        );

        let bot_left = background_rect.bot_left + border_size;
        let mut bar_size = bar_size - border_size.scale(2.0);
        bar_size.x *= ratio;
        let health_rect = Rect::from_bot_left(bot_left, bar_size);
        let health_primitive =
            DrawPrimitive::from_rect(health_rect, color, 0.0);

        [background_primitive, health_primitive]
    }
}

pub struct Weapon {
    pub range: f32,
    pub speed: f32,
    pub damage: f32,
    pub cooldown: f32,
}

impl Weapon {
    pub fn is_ready(&self) -> bool {
        self.cooldown >= 1.0 / self.speed
    }
}

pub struct Animator {
    pub rect: Rect,
    pub flip: bool,
    current_animation: &'static str,
    animation_to_sprite: HashMap<&'static str, AnimatedSprite>,
}

impl Animator {
    pub fn new(rect: Rect, default_sprite: AnimatedSprite) -> Self {
        let mut animation_to_sprite = HashMap::new();
        animation_to_sprite.insert("default", default_sprite);

        Self {
            rect: rect,
            flip: false,
            current_animation: "default",
            animation_to_sprite,
        }
    }

    pub fn add(
        &mut self,
        animation: &'static str,
        sprite: AnimatedSprite,
    ) {
        self.animation_to_sprite.insert(animation, sprite);
    }

    pub fn play(&mut self, animation: &'static str) {
        self.current_animation = animation;
    }

    pub fn get_draw_primitive(&self) -> DrawPrimitive {
        let mut sprite = self
            .animation_to_sprite
            .get(self.current_animation)
            .unwrap()
            .get_current_frame();

        DrawPrimitive::from_sprite(
            Origin::BotCenter(Vec2::zeros()),
            sprite,
            None,
            self.flip,
            Texture::Sprite,
        )
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_to_sprite
            .get_mut(self.current_animation)
            .unwrap()
            .update(dt);
    }
}

pub struct Text {
    pub draw_primitives: Vec<DrawPrimitive>,
}

impl Text {
    pub fn from_glyph_atlas(
        glyph_atlas: &GlyphAtlas,
        string: String,
        color: Color,
        scale: f32,
    ) -> Self {
        let mut draw_primitives = Vec::new();
        let mut position = Vec2::zeros();
        for (_, c) in string.char_indices() {
            let glyph = glyph_atlas.get_glyph(c);
            let sprite = Sprite {
                x: glyph.x,
                y: glyph.y,
                w: glyph.metrics.width as f32,
                h: glyph.metrics.height as f32,
                scale,
            };
            let mut primitive_position = position;
            primitive_position.x += glyph.metrics.xmin as f32 * scale;
            primitive_position.y += glyph.metrics.ymin as f32 * scale;
            let mut primitive = DrawPrimitive::from_sprite(
                Origin::BotLeft(Vec2::zeros()),
                sprite,
                Some(color),
                false,
                Texture::Glyph,
            )
            .translate(primitive_position);
            draw_primitives.push(primitive);

            position.x += glyph.metrics.advance_width * scale;
            position.y += glyph.metrics.advance_height * scale;
        }

        Self { draw_primitives }
    }
}
