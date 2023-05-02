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
    pub animator: Option<Animator>,
}

pub struct Kinematic {
    pub max_speed: f32,
    pub speed: f32,
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

pub struct Animator {
    pub flip: bool,
    current_animation: &'static str,
    animation_to_sprite: HashMap<&'static str, AnimatedSprite>,
}

impl Animator {
    pub fn new(default_sprite: AnimatedSprite) -> Self {
        let mut animation_to_sprite = HashMap::new();
        animation_to_sprite.insert("default", default_sprite);

        Self {
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

    pub fn get_sprite(&self) -> Sprite {
        let mut sprite = self
            .animation_to_sprite
            .get(self.current_animation)
            .unwrap()
            .get_current_frame();

        sprite.flip = self.flip;

        sprite
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_to_sprite
            .get_mut(self.current_animation)
            .unwrap()
            .update(dt);
    }
}
