#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::graphics::*;
use crate::vec::*;
use std::collections::HashMap;
use std::fs;

#[repr(u64)]
#[derive(Debug)]
pub enum Flag {
    Dead = 1 << 0,
    Player = 1 << 1,
}

#[derive(Clone, Debug)]
pub enum Behaviour {
    Player,
    Rat {
        min_jump_distance: f32,
        max_jump_distance: f32,
    },
}

#[derive(Clone)]
pub struct Entity {
    pub flags: u64,
    pub behaviour: Behaviour,
    pub position: Vec2<f32>,
    collider: Rect,

    pub move_speed: f32,

    pub jump_speed: f32,
    pub jump_period: f32,
    last_jump_time: f32,

    pub velocity: Vec2<f32>,

    max_health: f32,
    current_health: f32,

    melee_weapon: Option<MeleeWeapon>,
    range_weapon: Option<RangeWeapon>,

    pub animator: Option<Animator>,

    pub score: u32,
}

impl Entity {
    pub fn new(
        is_player: bool,
        behaviour: Behaviour,
        position: Vec2<f32>,
        collider: Rect,
        move_speed: f32,
        jump_speed: f32,
        jump_period: f32,
        max_health: f32,
        melee_weapon: Option<MeleeWeapon>,
        range_weapon: Option<RangeWeapon>,
        animator: Option<Animator>,
    ) -> Self {
        let flags = Flag::Player as u64 * is_player as u64;

        Self {
            flags,
            behaviour,
            position,
            collider,
            move_speed,
            jump_speed,
            jump_period,
            last_jump_time: -jump_period,
            velocity: Vec2::zeros(),
            max_health,
            current_health: max_health,
            melee_weapon,
            range_weapon,
            animator,
            score: 0,
        }
    }

    pub fn get_collider(&self) -> Rect {
        self.collider.with_bot_center(self.position)
    }

    pub fn get_center(&self) -> Vec2<f32> {
        self.get_collider().get_center()
    }

    pub fn receive_damage(&mut self, value: f32) {
        self.current_health -= value;
        if self.current_health <= 0.0 {
            self.set_flag(Flag::Dead);
        }
    }

    pub fn try_receive_bullet_damage(&mut self, bullet: &Bullet) -> bool {
        let self_collider = self.get_collider();
        let bullet_collider = bullet.get_collider();
        if self_collider.collide_with_rect(bullet_collider) {
            self.receive_damage(bullet.damage);
            return true;
        }

        false
    }

    pub fn try_receive_melee_attack_damage(
        &mut self,
        melee_attack: &MeleeAttack,
    ) -> bool {
        let self_collider = self.get_collider();
        let attack_collider = melee_attack.get_collider();
        if self_collider.collide_with_rect(attack_collider) {
            self.receive_damage(melee_attack.damage);
            return true;
        }

        false
    }

    pub fn get_health_ratio(&self) -> f32 {
        self.current_health / self.max_health
    }

    pub fn immediate_step(&mut self, direction: f32, dt: f32) {
        self.position.x +=
            direction.clamp(-1.0, 1.0) * self.move_speed * dt;
    }

    pub fn jump_to(&mut self, target: Vec2<f32>, time: f32) {
        self.velocity +=
            (target - self.position).with_len(self.jump_speed);
        self.last_jump_time = time;
    }

    pub fn jump_at_angle(&mut self, angle: f32, time: f32) {
        let target = self.position + Vec2::new(angle.cos(), angle.sin());
        self.jump_to(target, time);
    }

    pub fn attack_by_melee(
        &mut self,
        time: f32,
        attack_delay: Option<f32>,
    ) -> MeleeAttack {
        let weapon = self.melee_weapon.as_mut().unwrap();
        weapon.last_attack_time = time;

        let collider = Rect::from_center(
            weapon.pivot,
            Vec2::new(weapon.length, weapon.length),
        );

        let attack_delay = if let Some(delay) = attack_delay {
            delay
        } else {
            weapon.attack_duration
        };

        MeleeAttack::new(
            self.position,
            collider,
            weapon.damage,
            attack_delay,
            self.check_flag(Flag::Player),
        )
    }

    pub fn attack_by_range(
        &mut self,
        target: Vec2<f32>,
        time: f32,
    ) -> Bullet {
        let weapon = self.range_weapon.as_mut().unwrap();
        weapon.last_attack_time = time;

        let pivot = self.position + weapon.pivot;
        let direction = target - pivot;
        let position = pivot + direction.with_len(weapon.length);
        let collider =
            Rect::from_center(Vec2::zeros(), Vec2::new(0.2, 0.2));
        let velocity = direction.with_len(weapon.bullet_speed);

        Bullet::new(
            position,
            collider,
            velocity,
            weapon.bullet_damage,
            self.check_flag(Flag::Player),
        )
    }

    pub fn check_if_on_floor(&self, floor_y: f32) -> bool {
        (self.position.y - floor_y).abs() < 1e-4
    }

    pub fn check_if_attacking(&self, time: f32) -> bool {
        let mut is_attacking = if let Some(weapon) = self.melee_weapon {
            weapon.is_attacking(time)
        } else {
            false
        };

        is_attacking |= if let Some(weapon) = self.range_weapon {
            weapon.is_attacking(time)
        } else {
            false
        };

        is_attacking
    }

    pub fn check_if_cooling_down(&self, time: f32) -> bool {
        let mut is_attacking = if let Some(weapon) = self.melee_weapon {
            weapon.is_cooling_down(time)
        } else {
            false
        };

        is_attacking |= if let Some(weapon) = self.range_weapon {
            weapon.is_cooling_down(time)
        } else {
            false
        };

        is_attacking
    }

    pub fn check_if_can_jump(&self, floor_y: f32, time: f32) -> bool {
        !self.check_if_attacking(time)
            && !self.check_if_cooling_down(time)
            && self.check_if_on_floor(floor_y)
            && (time - self.last_jump_time) >= self.jump_period
    }

    pub fn check_if_can_step(&self, floor_y: f32, time: f32) -> bool {
        !self.check_if_attacking(time)
            && !self.check_if_cooling_down(time)
            && self.check_if_on_floor(floor_y)
    }

    pub fn check_if_can_reach_by_melee(
        &self,
        target: Rect,
        time: f32,
    ) -> bool {
        if let Some(weapon) = self.melee_weapon {
            let collider = Rect::from_center(
                weapon.pivot + self.position,
                Vec2::new(weapon.length, weapon.length),
            );
            !self.check_if_attacking(time)
                && !self.check_if_cooling_down(time)
                && collider.collide_with_rect(target)
        } else {
            false
        }
    }

    pub fn check_if_can_reach_by_range(
        &self,
        target: Vec2<f32>,
        time: f32,
    ) -> bool {
        if let Some(weapon) = self.range_weapon {
            !self.check_if_attacking(time)
                && !self.check_if_cooling_down(time)
        } else {
            false
        }
    }

    pub fn check_flag(&self, flag: Flag) -> bool {
        (self.flags & flag as u64) != 0
    }

    pub fn set_flag(&mut self, flag: Flag) {
        self.flags |= flag as u64
    }

    pub fn update_kinematic(
        &mut self,
        gravity: f32,
        floor_collider: Rect,
        dt: f32,
    ) {
        let floor_y = floor_collider.get_y_min();
        let was_on_floor = self.check_if_on_floor(floor_y);
        self.position += self.velocity.scale(dt);
        self.position.y = self.position.y.max(floor_y);
        let is_on_floor = self.check_if_on_floor(floor_y);

        if is_on_floor {
            if !was_on_floor {
                self.velocity.x = 0.0;
            }
            self.velocity.y = 0.0;
        } else {
            self.velocity.y -= gravity * dt;
        }

        let self_collider = self.get_collider();
        let left_offset =
            floor_collider.get_x_min() - self_collider.get_x_min();
        if left_offset > 0.0 {
            self.position.x += left_offset
        }

        let right_offset =
            self_collider.get_x_max() - floor_collider.get_x_max();
        if right_offset > 0.0 {
            self.position.x -= right_offset;
        }

        let up_offset =
            self_collider.get_y_max() - floor_collider.get_y_max();
        if up_offset > 0.0 {
            self.position.y -= up_offset;
            self.velocity.y = 0.0;
        }
    }

    pub fn update_animator(&mut self, dt: f32) {
        if let Some(animator) = self.animator.as_mut() {
            animator.update(dt);
        }
    }

    pub fn play_animation(&mut self, animation_type: AnimationType) {
        let animator = self.animator.as_mut().unwrap();
        animator.play(animation_type);
    }
}

#[derive(Clone, Copy)]
pub struct RangeWeapon {
    pub pivot: Vec2<f32>,
    pub length: f32,
    pub last_attack_time: f32,
    pub attack_duration: f32,
    pub attack_cooldown: f32,
    pub bullet_speed: f32,
    pub bullet_damage: f32,
}

impl RangeWeapon {
    pub fn new(
        pivot: Vec2<f32>,
        length: f32,
        attack_duration: f32,
        attack_cooldown: f32,
        bullet_speed: f32,
        bullet_damage: f32,
    ) -> Self {
        Self {
            pivot,
            length,
            last_attack_time: -(attack_duration + attack_cooldown),
            attack_duration,
            attack_cooldown,
            bullet_speed,
            bullet_damage,
        }
    }

    pub fn is_attacking(&self, time: f32) -> bool {
        (time - self.last_attack_time) < self.attack_duration
    }

    pub fn is_cooling_down(&self, time: f32) -> bool {
        !self.is_attacking(time)
            && (time - self.last_attack_time)
                < self.attack_duration + self.attack_cooldown
    }
}

#[derive(Clone, Copy)]
pub struct MeleeWeapon {
    pub pivot: Vec2<f32>,
    pub length: f32,
    pub last_attack_time: f32,
    pub attack_duration: f32,
    pub attack_cooldown: f32,
    pub damage: f32,
}

impl MeleeWeapon {
    pub fn new(
        pivot: Vec2<f32>,
        length: f32,
        attack_duration: f32,
        attack_cooldown: f32,
        damage: f32,
    ) -> Self {
        Self {
            pivot,
            length,
            last_attack_time: -(attack_duration + attack_cooldown),
            attack_duration,
            attack_cooldown,
            damage,
        }
    }

    pub fn is_attacking(&self, time: f32) -> bool {
        (time - self.last_attack_time) < self.attack_duration
    }

    pub fn is_cooling_down(&self, time: f32) -> bool {
        !self.is_attacking(time)
            && (time - self.last_attack_time)
                < self.attack_duration + self.attack_cooldown
    }
}

#[derive(Clone, Copy)]
pub struct Bullet {
    pub position: Vec2<f32>,
    collider: Rect,
    pub velocity: Vec2<f32>,
    pub damage: f32,
    pub is_player_friendly: bool,
}

impl Bullet {
    pub fn new(
        position: Vec2<f32>,
        collider: Rect,
        velocity: Vec2<f32>,
        damage: f32,
        is_player_friendly: bool,
    ) -> Self {
        Self {
            position,
            collider,
            velocity,
            damage,
            is_player_friendly,
        }
    }

    pub fn get_collider(&self) -> Rect {
        self.collider.with_center(self.position)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MeleeAttack {
    pub position: Vec2<f32>,
    collider: Rect,
    pub damage: f32,
    pub delay: f32,
    pub is_player_friendly: bool,
}

impl MeleeAttack {
    pub fn new(
        position: Vec2<f32>,
        collider: Rect,
        damage: f32,
        delay: f32,
        is_player_friendly: bool,
    ) -> Self {
        Self {
            position,
            collider,
            damage,
            delay,
            is_player_friendly,
        }
    }

    pub fn get_collider(&self) -> Rect {
        self.collider.with_center(self.position)
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

pub struct Spawner {
    position: Vec2<f32>,
    spawn_period: f32,
    n_to_spawn: usize,
    entity_to_spawn: Entity,
    countdown: f32,
}

impl Spawner {
    pub fn new(
        position: Vec2<f32>,
        spawn_period: f32,
        n_to_spawn: usize,
        entity_to_spawn: Entity,
    ) -> Self {
        Self {
            position,
            spawn_period,
            n_to_spawn,
            entity_to_spawn,
            countdown: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) -> Option<Entity> {
        let entity = if (self.countdown <= 0.0) && self.n_to_spawn > 0 {
            self.countdown += self.spawn_period;
            self.n_to_spawn -= 1;
            let mut entity = self.entity_to_spawn.clone();
            entity.position = self.position;

            Some(entity)
        } else {
            None
        };

        self.countdown -= dt;
        entity
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum AnimationType {
    Default_,
    Idle,
    Move,
    MeleeAttack,
    Jump,
    Hurt,
    Die,
}

#[derive(Clone)]
pub struct Animator {
    pub flip: bool,
    pub animation_type: AnimationType,
    animation_to_sprite: HashMap<AnimationType, AnimatedSprite>,
}

impl Animator {
    pub fn new(default_sprite: AnimatedSprite) -> Self {
        let mut animation_to_sprite = HashMap::new();
        animation_to_sprite
            .insert(AnimationType::Default_, default_sprite);

        Self {
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
        if self.animation_type != animation_type {
            self.animation_to_sprite
                .get_mut(&animation_type)
                .unwrap()
                .reset();
        }

        self.animation_type = animation_type;
    }

    pub fn get_draw_primitive(&self, origin: Origin) -> DrawPrimitive {
        let mut sprite = self
            .animation_to_sprite
            .get(&self.animation_type)
            .unwrap()
            .get_current_frame();

        DrawPrimitive::from_sprite(
            Space::World,
            origin,
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
        font_size: u32,
        color: Color,
        scale: f32,
    ) -> Self {
        let mut draw_primitives = Vec::new();
        let mut cursor_position = Vec2::zeros();
        for (_, c) in string.char_indices() {
            let glyph = glyph_atlas.get_glyph(c, font_size);
            let sprite = Sprite {
                x: glyph.x,
                y: glyph.y,
                w: glyph.metrics.width as f32,
                h: glyph.metrics.height as f32,
                scale,
            };
            let mut primitive_position = cursor_position;
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

            cursor_position.x += glyph.metrics.advance_width * scale;
            cursor_position.y += glyph.metrics.advance_height * scale;
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
            Origin::TopLeft(p) => {
                p + Vec2::new(0.0, bot_left.y - top_right.y)
            }
            Origin::LeftCenter(p) => {
                p + Vec2::new(0.0, 0.5 * (bot_left.y - top_right.y))
            }
            Origin::RightCenter(p) => {
                p + Vec2::new(
                    bot_left.x - top_right.x,
                    0.5 * (bot_left.y - top_right.y),
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
