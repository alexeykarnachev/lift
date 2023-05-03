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

    pub fn get_draw_primitive(
        &self,
        position: Vec2<f32>,
    ) -> DrawPrimitive {
        let mut sprite = self
            .animation_to_sprite
            .get(self.current_animation)
            .unwrap()
            .get_current_frame();

        DrawPrimitive::from_sprite(sprite, position, self.flip)
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_to_sprite
            .get_mut(self.current_animation)
            .unwrap()
            .update(dt);
    }
}
