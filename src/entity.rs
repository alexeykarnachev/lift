#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::graphics::*;
use crate::vec::*;
use std::collections::HashMap;
use std::fs;

#[repr(u32)]
#[derive(Debug)]
pub enum EntityState {
    Idle = 1 << 0,
    Move = 1 << 1,
    Jump = 1 << 2,
    Attack = 1 << 3,
    Dead = 1 << 4,
}

#[derive(Clone)]
pub struct Entity {
    pub states: u32,
    pub position: Vec2<f32>,
    pub collider: Option<Rect>,
    pub kinematic: Option<Kinematic>,
    pub health: Option<Health>,
    pub weapon: Option<Weapon>,
    pub bullet: Option<Bullet>,
    pub draw_primitive: Option<DrawPrimitive>,
    pub animator: Option<Animator>,
    pub text: Option<Text>,
}

impl Entity {
    pub fn new(position: Vec2<f32>) -> Self {
        Self {
            states: EntityState::Idle as u32,
            position,
            collider: None,
            kinematic: None,
            health: None,
            weapon: None,
            bullet: None,
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

    pub fn try_shoot(
        &mut self,
        target: Vec2<f32>,
        time: f32,
        is_player_friendly: bool,
    ) -> Option<Entity> {
        let weapon = self.weapon.as_mut().unwrap();
        let can_shoot =
            (time - weapon.last_shoot_time) >= weapon.shoot_period;
        let bullet = if can_shoot {
            weapon.last_shoot_time = time;
            let position = self.position;
            let velocity =
                (target - position).with_len(weapon.bullet_speed);
            let mut bullet = Entity::new(position);
            bullet.bullet = Some(Bullet {
                damage: weapon.bullet_damage,
                is_player_friendly,
            });
            bullet.kinematic = Some(Kinematic::with_velocity(velocity));
            bullet.draw_primitive = Some(DrawPrimitive::from_rect(
                Rect::from_center(Vec2::zeros(), Vec2::new(0.1, 0.1)),
                Space::World,
                Color::red(1.0),
            ));

            Some(bullet)
        } else {
            None
        };

        bullet
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

    pub fn get_text_rect(&self) -> Rect {
        let text = self.text.as_ref().unwrap();
        let first = text.draw_primitives[0].rect;
        let last =
            text.draw_primitives[text.draw_primitives.len() - 1].rect;

        Rect {
            bot_left: first.bot_left,
            top_right: last.top_right,
        }
        .translate(self.position)
    }

    pub fn set_text_color(&mut self, color: Color) {
        self.text.as_mut().unwrap().change_color(color);
    }
}

#[derive(Clone, Copy)]
pub struct Kinematic {
    pub move_speed: f32,
    pub jump_speed: f32,
    pub velocity: Vec2<f32>,
}

impl Kinematic {
    pub fn new(move_speed: f32, jump_speed: f32) -> Self {
        Self {
            move_speed,
            jump_speed,
            velocity: Vec2::zeros(),
        }
    }

    pub fn with_velocity(velocity: Vec2<f32>) -> Self {
        Self {
            move_speed: 0.0,
            jump_speed: 0.0,
            velocity,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { max, current: max }
    }

    pub fn receive_damage(&mut self, value: f32) {
        self.current = (self.current - value).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

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
            Space::World,
            Color::gray(0.2, 1.0),
        );

        let bot_left = background_rect.bot_left + border_size;
        let mut bar_size = bar_size - border_size.scale(2.0);
        bar_size.x *= ratio;
        let health_rect = Rect::from_bot_left(bot_left, bar_size);
        let health_primitive =
            DrawPrimitive::from_rect(health_rect, Space::World, color);

        [background_primitive, health_primitive]
    }
}

#[derive(Clone, Copy)]
pub struct Weapon {
    pub last_shoot_time: f32,
    pub shoot_period: f32,
    pub bullet_speed: f32,
    pub bullet_damage: f32,
}

impl Weapon {
    pub fn new(
        shoot_period: f32,
        bullet_speed: f32,
        bullet_damage: f32,
    ) -> Self {
        Self {
            last_shoot_time: -shoot_period,
            shoot_period,
            bullet_speed,
            bullet_damage,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bullet {
    pub damage: f32,
    pub is_player_friendly: bool,
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
    pub draw_primitives: Vec<DrawPrimitive>,
}

impl Text {
    pub fn from_glyph_atlas(
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

        Self { draw_primitives }
    }

    pub fn change_color(&mut self, color: Color) {
        for primitive in self.draw_primitives.iter_mut() {
            primitive.color = Some(color);
        }
    }
}
