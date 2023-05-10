#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::graphics::*;
use crate::vec::*;
use std::collections::HashMap;
use std::fs;

/*
#[repr(u32)]
#[derive(Debug)]
pub enum EntityState {
    Idle = 1 << 0,
    Move = 1 << 1,
    Jump = 1 << 2,
    Attack = 1 << 3,
    Dead = 1 << 4,
}
*/

#[derive(Clone)]
pub struct Humanoid {
    pub position: Vec2<f32>,
    pub collider: Rect,

    pub move_speed: f32,
    pub jump_speed: f32,
    pub velocity: Vec2<f32>,

    pub max_health: f32,
    pub current_health: f32,

    pub weapon: Weapon,
}

impl Humanoid {
    pub fn new(
        position: Vec2<f32>,
        collider: Rect,
        move_speed: f32,
        jump_speed: f32,
        max_health: f32,
        weapon: Weapon,
    ) -> Self {
        Self {
            position,
            collider,
            move_speed,
            jump_speed,
            velocity: Vec2::zeros(),
            max_health,
            current_health: max_health,
            weapon,
        }
    }

    pub fn get_collider(&self) -> Rect {
        self.collider.with_bot_center(self.position)
    }

    pub fn try_shoot(
        &mut self,
        target: Vec2<f32>,
        time: f32,
        is_player_friendly: bool,
    ) -> Option<Bullet> {
        let weapon = &mut self.weapon;
        let can_shoot =
            (time - weapon.last_shoot_time) >= weapon.shoot_period;

        if !can_shoot {
            return None;
        }

        weapon.last_shoot_time = time;
        let pivot = self.position + weapon.pivot;
        let direction = target - pivot;
        let start_position = pivot + direction.with_len(weapon.length);
        let velocity = direction.with_len(weapon.bullet_speed);

        Some(Bullet::new(
            start_position,
            velocity,
            weapon.bullet_damage,
            weapon.bullet_max_travel_distance,
            is_player_friendly,
        ))
    }

    /*
    pub fn update_animator(&mut self, dt: f32) {
        use AnimationType::*;
        let animator = self.animator.as_mut().unwrap();
        match self.state {
            EntityState::Idle => animator.play(Idle),
            EntityState::Moving => animator.play(Move),
            EntityState::Jumpint => animator.play(Jump),
            EntityState::Attacking => animator.play(Attack),
            EntityState::Dead => animator.play(Die),
        }

        animator.update(dt);
    }
    */
}

#[derive(Clone, Copy)]
pub struct Weapon {
    pub pivot: Vec2<f32>,
    pub length: f32,
    pub last_shoot_time: f32,
    pub shoot_period: f32,
    pub bullet_speed: f32,
    pub bullet_damage: f32,
    pub bullet_max_travel_distance: f32,
}

impl Weapon {
    pub fn new(
        pivot: Vec2<f32>,
        length: f32,
        shoot_period: f32,
        bullet_speed: f32,
        bullet_damage: f32,
        bullet_max_travel_distance: f32,
    ) -> Self {
        Self {
            pivot,
            length,
            last_shoot_time: -shoot_period,
            shoot_period,
            bullet_speed,
            bullet_damage,
            bullet_max_travel_distance,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bullet {
    pub position: Vec2<f32>,
    pub start_position: Vec2<f32>,
    pub velocity: Vec2<f32>,
    pub damage: f32,
    pub max_travel_distance: f32,
    pub is_player_friendly: bool,
}

impl Bullet {
    pub fn new(
        start_position: Vec2<f32>,
        velocity: Vec2<f32>,
        damage: f32,
        max_travel_distance: f32,
        is_player_friendly: bool,
    ) -> Self {
        Self {
            position: start_position,
            start_position,
            velocity,
            damage,
            max_travel_distance,
            is_player_friendly,
        }
    }
}

pub struct Shaft {
    collider: Rect,
}

impl Shaft {
    pub fn new(width: f32, height: f32) -> Self {
        let collider =
            Rect::from_bot_center(Vec2::zeros(), Vec2::new(width, height));

        Self { collider }
    }

    pub fn get_collider(&self) -> Rect {
        self.collider
    }
}

pub struct Floor {
    pub y: f32,
    pub idx: usize,
    collider: Rect,
}

impl Floor {
    pub fn new(y: f32, idx: usize, width: f32, height: f32) -> Self {
        let collider =
            Rect::from_bot_center(Vec2::zeros(), Vec2::new(width, height));

        Self { y, idx, collider }
    }

    pub fn get_collider(&self) -> Rect {
        self.collider.translate(Vec2::new(0.0, self.y))
    }
}

pub struct Lift {
    pub y: f32,
    pub speed: f32,

    collider: Rect,
}

impl Lift {
    pub fn new(y: f32, width: f32, height: f32, speed: f32) -> Self {
        let collider =
            Rect::from_bot_center(Vec2::zeros(), Vec2::new(width, height));

        Self { y, speed, collider }
    }

    pub fn get_collider(&self) -> Rect {
        self.collider.translate(Vec2::new(0.0, self.y))
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum AnimationType {
    Default_,
    Idle,
    Move,
    Attack,
    Hurt,
    Die,
}

#[derive(Clone)]
pub struct Animator {
    pub rect: Rect,
    pub flip: bool,
    animation_type: AnimationType,
    animation_to_sprite: HashMap<AnimationType, AnimatedSprite>,
}

impl Animator {
    pub fn new(rect: Rect, default_sprite: AnimatedSprite) -> Self {
        let mut animation_to_sprite = HashMap::new();
        animation_to_sprite
            .insert(AnimationType::Default_, default_sprite);

        Self {
            rect,
            flip: false,
            animation_type: AnimationType::Default_,
            animation_to_sprite,
        }
    }

    pub fn add(
        &mut self,
        animation_type: AnimationType,
        sprite: AnimatedSprite,
    ) {
        self.animation_to_sprite.insert(animation_type, sprite);
    }

    pub fn play(&mut self, animation_type: AnimationType) {
        self.animation_type = animation_type;
    }

    pub fn get_draw_primitive(&self) -> DrawPrimitive {
        let mut sprite = self
            .animation_to_sprite
            .get(&self.animation_type)
            .unwrap()
            .get_current_frame();

        DrawPrimitive::from_sprite(
            Space::World,
            Origin::BotCenter(Vec2::zeros()),
            sprite,
            None,
            self.flip,
            Texture::Sprite,
        )
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_to_sprite
            .get_mut(&self.animation_type)
            .unwrap()
            .update(dt);
    }
}

#[derive(Clone)]
pub struct Text {
    pub position: Vec2<f32>,
    draw_primitives: Vec<DrawPrimitive>,
}

impl Text {
    pub fn new(
        position: Vec2<f32>,
        glyph_atlas: &GlyphAtlas,
        space: Space,
        origin: Origin,
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
                space,
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

        let bot_left = draw_primitives[0].rect.bot_left;
        let top_right =
            draw_primitives[draw_primitives.len() - 1].rect.top_right;
        let offset = match origin {
            Origin::Center(p) => p + (bot_left - top_right).scale(0.5),
            Origin::BotCenter(p) => {
                p + Vec2::new(0.5 * (bot_left.x - top_right.x), 0.0)
            }
            Origin::BotLeft(p) => p,
            Origin::LeftCenter(p) => {
                p + Vec2::new(0.0, 0.5 * (-bot_left.y + top_right.y))
            }
            Origin::RightCenter(p) => {
                p + Vec2::new(
                    bot_left.x - top_right.x,
                    0.5 * (-bot_left.y + top_right.y),
                )
            }
        };

        let draw_primitives = draw_primitives
            .iter_mut()
            .map(|p| p.translate(offset))
            .collect();

        Self {
            position,
            draw_primitives,
        }
    }

    pub fn get_bound_rect(&self) -> Rect {
        let first = self.draw_primitives[0].rect;
        let last =
            self.draw_primitives[self.draw_primitives.len() - 1].rect;

        Rect {
            bot_left: first.bot_left,
            top_right: last.top_right,
        }
        .translate(self.position)
    }

    pub fn get_draw_primitives(&self) -> Vec<DrawPrimitive> {
        self.draw_primitives
            .iter()
            .map(|p| p.translate(self.position))
            .collect()
    }

    pub fn set_color(&mut self, color: Color) {
        for primitive in self.draw_primitives.iter_mut() {
            primitive.color = Some(color);
        }
    }
}
