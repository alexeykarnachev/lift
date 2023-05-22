#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::graphics::*;
use crate::level::Collider;
use crate::prefabs::create_rat;
use crate::utils::frand;
use crate::vec::*;
use std::collections::HashMap;
use std::fs;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Behaviour {
    Player,
    Rat,
    Bat,
    Spawner,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Initial,
    Idle,
    Running,
    Dashing,
    Attacking,
    Falling,
    Jumping,
    Healing,
    Dead,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Orientation {
    Left,
    Right,
}

#[derive(Clone)]
pub struct Entity {
    pub id: i32,
    pub time: f32,
    pub state: State,
    pub is_on_floor: bool,
    pub is_on_ceil: bool,

    pub behaviour: Option<Behaviour>,
    pub position: Vec2<f32>,
    pub orientation: Orientation,
    pub apply_gravity: bool,
    pub collider: Option<Rect>,

    pub move_speed: f32,

    pub jump_speed: f32,
    pub jump_period: f32,
    pub last_jump_time: f32,

    pub velocity: Vec2<f32>,

    pub max_health: f32,
    pub current_health: f32,
    pub stamina: Option<Stamina>,
    pub last_received_damage_time: f32,

    pub knockback_resist: f32,

    pub dashing: Option<Dashing>,
    pub healing: Option<Healing>,
    pub weapons: Vec<Weapon>,
    pub weapon_idx: usize,

    pub light: Option<Light>,
    pub animator: Option<Animator>,
    pub spawner: Option<Spawner>,
    pub effect: u32,

    pub score: u32,

    pub spawner_id: Option<usize>,
}

impl Entity {
    pub fn new(position: Vec2<f32>) -> Self {
        Self {
            id: -1,
            time: 0.0,
            state: State::Initial,
            is_on_floor: false,
            is_on_ceil: false,
            behaviour: None,
            position,
            orientation: Orientation::Right,
            apply_gravity: false,
            collider: None,
            move_speed: 0.0,
            jump_speed: 0.0,
            jump_period: 0.0,
            last_jump_time: -f32::INFINITY,
            velocity: Vec2::zeros(),
            max_health: 0.0,
            current_health: 0.0,
            stamina: None,
            knockback_resist: 0.0,
            last_received_damage_time: -f32::INFINITY,
            dashing: None,
            healing: None,
            weapons: vec![],
            weapon_idx: 0,
            light: None,
            animator: None,
            spawner: None,
            effect: 0,
            score: 0,
            spawner_id: None,
        }
    }

    pub fn get_collider(&self) -> Option<Rect> {
        if let Some(collider) = self.collider {
            return Some(collider.translate(self.position));
        }

        None
    }

    pub fn get_center(&self) -> Vec2<f32> {
        if let Some(collider) = self.get_collider() {
            return collider.get_center();
        }

        self.position
    }

    pub fn get_top_left(&self) -> Vec2<f32> {
        if let Some(collider) = self.get_collider() {
            return collider.get_top_left();
        }

        self.position
    }

    pub fn get_bot_center(&self) -> Vec2<f32> {
        if let Some(collider) = self.get_collider() {
            return collider.get_bot_center();
        }

        self.position
    }

    pub fn get_time_since_last_received_damage(&self) -> f32 {
        self.time - self.last_received_damage_time
    }

    pub fn get_time_since_last_jump(&self) -> f32 {
        self.time - self.last_jump_time
    }

    pub fn get_light(&self) -> Option<Light> {
        if let Some(mut light) = self.light {
            light.position += self.position;
            return Some(light);
        };

        None
    }

    pub fn set_orientation(&mut self, is_right: bool) {
        use Orientation::*;

        if is_right {
            self.orientation = Right;
        } else {
            self.orientation = Left;
        }
    }

    pub fn set_weapon(&mut self, weapon_idx: usize) {
        if weapon_idx >= self.weapons.len() {
            panic!(
                "Can't set weapon with idx: {:?}.
                   There are {:?} weapons on this entity",
                weapon_idx,
                self.weapons.len()
            );
        }
        self.weapon_idx = weapon_idx;
    }

    pub fn receive_damage(&mut self, value: f32) {
        self.current_health -= value;
        self.last_received_damage_time = self.time;
    }

    pub fn try_receive_bullet_damage(&mut self, bullet: &Bullet) -> bool {
        if self.check_if_dashing() {
            return false;
        }

        let bullet_collider = bullet.get_collider();
        if let Some(self_collider) = self.get_collider() {
            if self_collider.collide_with_rect(bullet_collider) {
                self.receive_damage(bullet.damage);
                return true;
            }
        }

        false
    }

    pub fn collide_with_attack(&self, attack: &Attack) -> bool {
        if self.check_if_dashing() {
            return false;
        }

        let attack_collider = attack.get_collider();
        if let Some(self_collider) = self.get_collider() {
            if self_collider.collide_with_rect(attack_collider) {
                return true;
            }
        }

        false
    }

    pub fn get_health_ratio(&self) -> f32 {
        self.current_health / self.max_health
    }

    pub fn get_stamina_ratio(&self) -> f32 {
        self.stamina.unwrap().get_ratio()
    }

    pub fn immediate_step(&mut self, direction: Vec2<f32>, dt: f32) {
        let step = direction.norm().with_len(self.move_speed * dt);
        self.position += step;
    }

    pub fn jump_to(&mut self, target: Vec2<f32>, jump_speed: Option<f32>) {
        let jump_speed = if let Some(jump_speed) = jump_speed {
            jump_speed
        } else {
            self.jump_speed
        };
        self.velocity += (target - self.position).with_len(jump_speed);
        self.last_jump_time = self.time;
    }

    pub fn jump_at_angle(&mut self, angle: f32, jump_speed: Option<f32>) {
        let target = self.position + Vec2::new(angle.cos(), angle.sin());
        self.jump_to(target, jump_speed);
    }

    pub fn force_start_healing(&mut self) {
        self.healing.as_mut().unwrap().force_start();
    }

    pub fn force_stop_healing(&mut self) {
        self.healing.as_mut().unwrap().force_stop();
    }

    pub fn force_start_dashing(&mut self) {
        let dashing = self.dashing.as_mut().unwrap();
        if let Some(stamina) = self.stamina.as_mut() {
            stamina.sub(dashing.stamina_cost);
        }

        dashing.force_start();
    }

    pub fn force_attack(&mut self) -> Attack {
        use Orientation::*;

        let weapon = &mut self.weapons[self.weapon_idx];
        weapon.last_attack_time = self.time;
        let collider = weapon.get_collider(self.orientation);

        if let Some(stamina) = self.stamina.as_mut() {
            stamina.sub(weapon.stamina_cost);
        }

        Attack::new(
            self.position,
            collider,
            weapon.damage,
            weapon.attack_duration,
            self.check_if_player(),
        )
    }

    pub fn check_if_player(&self) -> bool {
        if let Some(Behaviour::Player) = self.behaviour {
            return true;
        }
        false
    }

    pub fn check_if_dead(&self) -> bool {
        self.current_health <= 0.0
    }

    pub fn check_if_attacking(&self) -> bool {
        self.weapons[self.weapon_idx].is_attacking(self.time)
    }

    pub fn check_if_cooling_down(&self) -> bool {
        self.weapons[self.weapon_idx].is_cooling_down(self.time)
    }

    pub fn check_if_dashing(&self) -> bool {
        if let Some(dashing) = self.dashing {
            dashing.is_started
        } else {
            false
        }
    }

    pub fn check_if_healing(&self) -> bool {
        if let Some(healing) = self.healing {
            healing.is_started
        } else {
            false
        }
    }

    pub fn check_if_enough_stamina_for_attack(&self) -> bool {
        let weapon = &self.weapons[self.weapon_idx];
        if let Some(stamina) = self.stamina.as_ref() {
            if stamina.current < weapon.stamina_cost {
                return false;
            }
        }

        true
    }

    pub fn check_if_enough_stamina_for_dashing(&self) -> bool {
        if let (Some(dashing), Some(stamina)) =
            (self.dashing, self.stamina)
        {
            if stamina.current < dashing.stamina_cost {
                return false;
            }
        }

        true
    }

    pub fn check_if_jump_ready(&self) -> bool {
        self.is_on_floor
            && self.get_time_since_last_jump() >= self.jump_period
    }

    pub fn check_if_weapon_ready(&self) -> bool {
        !self.check_if_attacking() && !self.check_if_cooling_down()
    }

    pub fn check_if_dashing_ready(&self) -> bool {
        if let Some(dashing) = self.dashing {
            self.dashing.as_ref().unwrap().check_if_can_start()
        } else {
            false
        }
    }

    pub fn check_if_healing_ready(&self) -> bool {
        if let Some(healing) = self.healing {
            self.healing.as_ref().unwrap().check_if_can_start()
        } else {
            false
        }
    }

    pub fn check_if_can_reach_by_weapon(&self, target: Rect) -> bool {
        let weapon = self.weapons[self.weapon_idx];
        let collider = weapon
            .get_collider(self.orientation)
            .translate(self.position);

        self.check_if_weapon_ready() && collider.collide_with_rect(target)
    }

    fn update_kinematic(
        &mut self,
        gravity: f32,
        friction: f32,
        colliders: &Vec<Collider>,
        dt: f32,
    ) {
        if !self.is_on_floor && self.apply_gravity {
            self.velocity.y -= gravity * dt;
        }
        self.position += self.velocity.scale(dt);
        self.velocity.x *= 1.0 - friction;

        let was_on_floor = self.is_on_floor;
        self.is_on_ceil = false;
        self.is_on_floor = false;
        if let Some(self_rect) = self.get_collider() {
            for collider in colliders {
                match collider {
                    Collider::Rigid(rect) => {
                        if rect.collide_with_rect(self_rect) {
                            let velocity_x;
                            let offset_x;
                            if rect.get_x_max() > self_rect.get_x_max() {
                                velocity_x = self.velocity.x.max(0.0);
                                offset_x = rect.get_x_min()
                                    - self_rect.get_x_max();
                            } else {
                                velocity_x = self.velocity.x.min(0.0);
                                offset_x = rect.get_x_max()
                                    - self_rect.get_x_min();
                            };

                            let velocity_y;
                            let offset_y;
                            let mut is_on_ceil = false;
                            let mut is_on_floor = false;
                            if rect.get_y_max() > self_rect.get_y_max() {
                                is_on_ceil = true;
                                velocity_y = self.velocity.y.min(0.0);
                                offset_y = rect.get_y_min()
                                    - self_rect.get_y_max();
                            } else {
                                is_on_floor = true;
                                if !was_on_floor {
                                    self.velocity.x = 0.0;
                                }
                                velocity_y = self.velocity.y.max(0.0);
                                offset_y = rect.get_y_max()
                                    - self_rect.get_y_min();
                            };

                            if offset_x.abs() < offset_y.abs() {
                                self.position.x += offset_x;
                                self.velocity.x = velocity_x;
                            } else {
                                self.position.y += offset_y;
                                self.velocity.y = velocity_y;
                                self.is_on_ceil = is_on_ceil;
                                self.is_on_floor = is_on_floor;
                            }
                        }
                    }
                };
            }
        }
    }

    fn update_dashing(&mut self, dt: f32) {
        use Orientation::*;

        if let Some(dashing) = self.dashing.as_mut() {
            let step = dashing.update(dt);
            self.position.x += match self.orientation {
                Left => -step,
                Right => step,
            }
        }
    }

    fn update_healing(&mut self, dt: f32) {
        let can_heal = !self.check_if_dead();
        if let Some(healing) = self.healing.as_mut() {
            if can_heal {
                self.current_health += healing.update(dt);
            }
        }
    }

    fn update_stamina(&mut self, dt: f32) {
        if let Some(stamina) = self.stamina.as_mut() {
            stamina.update(dt);
        }
    }

    fn update_animator(&mut self, dt: f32) {
        if let Some(animator) = self.animator.as_mut() {
            animator.update(dt);
            animator.flip = self.orientation == Orientation::Left;
        }
    }

    pub fn update_spawner(
        &mut self,
        dt: f32,
        sprite_atlas: &SpriteAtlas,
    ) -> Option<Entity> {
        let position = self.get_bot_center();
        if let Some(spawner) = self.spawner.as_mut() {
            if let Some(mut entity) =
                spawner.update(dt, position, sprite_atlas)
            {
                entity.spawner_id = Some(self.id as usize);
                return Some(entity);
            }
        };

        None
    }

    pub fn update(
        &mut self,
        gravity: f32,
        friction: f32,
        colliders: &Vec<Collider>,
        dt: f32,
    ) {
        self.time += dt;
        self.update_kinematic(gravity, friction, colliders, dt);
        self.update_dashing(dt);
        self.update_healing(dt);
        self.update_stamina(dt);
        self.update_animator(dt);
    }

    pub fn play_animation(&mut self, name: &'static str) {
        let animator = self.animator.as_mut().unwrap();
        animator.play(name);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dashing {
    speed: f32,
    duration: f32,
    cooldown: f32,
    time_since_start: f32,
    is_started: bool,
    pub stamina_cost: f32,
}

impl Dashing {
    pub fn new(
        speed: f32,
        duration: f32,
        cooldown: f32,
        stamina_cost: f32,
    ) -> Self {
        Self {
            speed,
            duration,
            cooldown,
            time_since_start: cooldown + duration,
            is_started: false,
            stamina_cost,
        }
    }

    pub fn check_if_can_start(&self) -> bool {
        !self.is_started
            && (self.time_since_start >= (self.cooldown + self.duration))
    }

    pub fn force_start(&mut self) {
        self.is_started = true;
        self.time_since_start = 0.0;
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        self.time_since_start += dt;

        let value = if !self.is_started {
            0.0
        } else {
            if self.time_since_start >= self.duration {
                self.is_started = false;
            }

            self.speed * dt
        };

        value
    }
}

#[derive(Clone, Copy)]
pub struct Stamina {
    max: f32,
    current: f32,
    regen: f32,
}

impl Stamina {
    pub fn new(max: f32, regen: f32) -> Self {
        Self {
            max,
            current: max,
            regen,
        }
    }

    pub fn get_ratio(&self) -> f32 {
        self.current / self.max
    }

    pub fn update(&mut self, dt: f32) {
        self.current = (self.current + dt * self.regen).min(self.max);
    }

    pub fn sub(&mut self, value: f32) {
        self.current = (self.current - value).max(0.0);
    }
}

#[derive(Clone, Copy)]
pub struct Healing {
    speed: f32,
    duration: f32,
    cooldown: f32,
    time_since_start: f32,
    is_started: bool,
}

impl Healing {
    pub fn new(speed: f32, duration: f32, cooldown: f32) -> Self {
        Self {
            speed,
            duration,
            cooldown,
            time_since_start: cooldown + duration,
            is_started: false,
        }
    }

    pub fn check_if_can_start(&self) -> bool {
        !self.is_started
            && (self.time_since_start >= (self.cooldown + self.duration))
    }

    pub fn force_start(&mut self) {
        self.is_started = true;
        self.time_since_start = 0.0;
    }

    pub fn force_stop(&mut self) {
        self.is_started = false;
        self.time_since_start = self.duration;
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        self.time_since_start += dt;

        let value = if !self.is_started {
            0.0
        } else {
            if self.time_since_start >= self.duration {
                self.is_started = false;
            }

            self.speed * dt
        };

        value
    }
}

#[derive(Clone, Copy)]
pub struct Weapon {
    collider: Rect,
    pub last_attack_time: f32,
    pub attack_duration: f32,
    pub attack_cooldown: f32,
    pub damage: f32,
    pub stamina_cost: f32,
}

impl Weapon {
    pub fn new(
        collider: Rect,
        attack_duration: f32,
        attack_cooldown: f32,
        damage: f32,
        stamina_cost: f32,
    ) -> Self {
        Self {
            collider,
            last_attack_time: -(attack_duration + attack_cooldown),
            attack_duration,
            attack_cooldown,
            damage,
            stamina_cost,
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

    pub fn get_collider(&self, orientation: Orientation) -> Rect {
        use Orientation::*;

        match orientation {
            Left => {
                let pivot = self.collider.get_center();
                self.collider.with_center(pivot.with_x(-pivot.x))
            }
            Right => self.collider,
        }
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
pub struct Attack {
    pub position: Vec2<f32>,
    collider: Rect,
    pub damage: f32,
    pub delay: f32,
    pub is_player_friendly: bool,
}

impl Attack {
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
        self.collider.translate(self.position)
    }
}

#[derive(Clone)]
pub struct Spawner {
    spawn_period: f32,
    n_to_spawn: u32,
    n_alive_max: u32,
    pub n_alive_current: u32,
    behaviour: Behaviour,
    countdown: f32,
    spawn_range_x: f32,
    spawn_range_y: f32,
}

impl Spawner {
    pub fn new(
        spawn_period: f32,
        n_to_spawn: u32,
        n_alive_max: u32,
        behaviour: Behaviour,
        spawn_range_x: f32,
        spawn_range_y: f32,
    ) -> Self {
        Self {
            spawn_period,
            n_to_spawn,
            n_alive_max,
            n_alive_current: 0,
            behaviour,
            countdown: 0.0,
            spawn_range_x,
            spawn_range_y,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        position: Vec2<f32>,
        sprite_atlas: &SpriteAtlas,
    ) -> Option<Entity> {
        let entity = if self.countdown <= 0.0
            && self.n_to_spawn > 0
            && self.n_alive_current < self.n_alive_max
        {
            self.countdown += self.spawn_period;
            self.n_to_spawn -= 1;
            self.n_alive_current += 1;
            let x = position.x
                + frand(-self.spawn_range_x, self.spawn_range_x);
            let y = position.y
                + frand(-self.spawn_range_y, self.spawn_range_y);
            let position = Vec2::new(x, y);

            let entity = match self.behaviour {
                Behaviour::Rat => create_rat(position, sprite_atlas),
                _ => {
                    panic!(
                        "Spawner for {:?} is not implemented",
                        self.behaviour
                    )
                }
            };

            Some(entity)
        } else {
            None
        };

        self.countdown = (self.countdown - dt).max(0.0);
        entity
    }
}

#[derive(Clone)]
pub struct Animator {
    pub flip: bool,
    pub animation: &'static str,
    animation_to_sprite: HashMap<&'static str, AnimatedSprite>,
}

impl Animator {
    pub fn new(default_sprite: AnimatedSprite) -> Self {
        let mut animation_to_sprite = HashMap::new();
        animation_to_sprite.insert("default", default_sprite);

        Self {
            flip: false,
            animation: "default",
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

    pub fn reset(&mut self) {
        self.animation_to_sprite
            .get_mut(self.animation)
            .unwrap()
            .reset();
    }

    pub fn play(&mut self, name: &'static str) {
        if self.animation != name {
            self.animation_to_sprite.get_mut(name).unwrap().reset();
        }

        self.animation = name;
    }

    pub fn get_draw_primitive(
        &self,
        position: Vec2<f32>,
        effect: u32,
    ) -> DrawPrimitive {
        let mut sprite = self
            .animation_to_sprite
            .get(self.animation)
            .unwrap()
            .get_current_frame();

        DrawPrimitive::from_sprite(
            SpaceType::WorldSpace,
            0.0,
            effect,
            position,
            sprite,
            None,
            self.flip,
            TextureType::SpriteTexture,
        )
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_to_sprite
            .get_mut(self.animation)
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
        space: SpaceType,
        alignment: Origin,
        string: String,
        font_size: u32,
        color: Color,
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
                origin: Origin::BotLeft,
            };
            let mut primitive_position = cursor_position;
            primitive_position.x += glyph.metrics.xmin as f32;
            primitive_position.y += glyph.metrics.ymin as f32;
            let mut primitive = DrawPrimitive::from_sprite(
                space,
                0.0,
                0,
                Vec2::zeros(),
                sprite,
                Some(color),
                false,
                TextureType::GlyphTexture,
            )
            .translate(primitive_position);
            draw_primitives.push(primitive);

            cursor_position.x += glyph.metrics.advance_width;
            cursor_position.y += glyph.metrics.advance_height;
        }

        let bot_left = draw_primitives[0].rect.bot_left;
        let top_right =
            draw_primitives[draw_primitives.len() - 1].rect.top_right;
        let offset = match alignment {
            Origin::Center => (bot_left - top_right).scale(0.5),
            Origin::BotCenter => {
                Vec2::new(0.5 * (bot_left.x - top_right.x), 0.0)
            }
            Origin::TopCenter => Vec2::new(
                0.5 * (bot_left.x - top_right.x),
                bot_left.y - top_right.y,
            ),
            Origin::BotLeft => Vec2::zeros(),
            Origin::TopLeft => Vec2::new(0.0, bot_left.y - top_right.y),
            Origin::TopRight => bot_left - top_right,
            Origin::LeftCenter => {
                Vec2::new(0.0, 0.5 * (bot_left.y - top_right.y))
            }
            Origin::RightCenter => Vec2::new(
                bot_left.x - top_right.x,
                0.5 * (bot_left.y - top_right.y),
            ),
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
