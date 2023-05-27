#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::graphics::*;
use crate::level::Collider;
use crate::prefabs::create_rat;
use crate::utils::*;
use crate::vec::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fs;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Behaviour {
    Player,
    Rat,
    Bat,
    RatNest,
    RatKing,
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
    Climbing,
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
    pub is_on_stair: bool,

    pub behaviour: Option<Behaviour>,
    pub position: Vec2<f32>,
    pub orientation: Orientation,
    pub apply_gravity: bool,
    pub collider: Option<Rect>,
    pub view_distance: f32,

    pub move_speed: f32,

    pub velocity: Vec2<f32>,

    pub max_health: f32,
    pub current_health: f32,
    pub stamina: Option<Stamina>,
    pub last_received_damage_time: f32,

    pub knockback_resist: f32,

    pub jumping: Option<Jumping>,
    pub dashing: Option<Dashing>,
    pub healing: Option<Healing>,
    pub weapons: Vec<Weapon>,
    pub weapon_idx: usize,

    pub light: Option<Light>,
    pub animator: Option<Animator>,
    pub spawner: Option<Spawner>,
    pub particles_emitter: ParticlesEmitter,
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
            is_on_stair: false,
            behaviour: None,
            position,
            orientation: Orientation::Right,
            apply_gravity: false,
            collider: None,
            view_distance: 0.0,
            move_speed: 0.0,
            velocity: Vec2::zeros(),
            max_health: 0.0,
            current_health: 0.0,
            stamina: None,
            knockback_resist: 0.0,
            last_received_damage_time: -f32::INFINITY,
            jumping: None,
            dashing: None,
            healing: None,
            weapons: vec![],
            weapon_idx: 0,
            light: None,
            animator: None,
            spawner: None,
            particles_emitter: ParticlesEmitter::empty(),
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

    pub fn get_light(&self) -> Option<Light> {
        if let Some(mut light) = self.light {
            light.position += self.position;
            return Some(light);
        };

        None
    }

    pub fn get_current_animation_cycle(&self) -> f32 {
        self.animator.as_ref().unwrap().get_current_cycle()
    }

    pub fn set_apply_gravity(&mut self, apply_gravity: bool) {
        self.apply_gravity = apply_gravity;
        self.velocity.y = 0.0;
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

    pub fn receive_attack(&mut self, attack: &Attack) {
        self.current_health -= attack.damage;
        self.last_received_damage_time = self.time;

        let mut angle = PI * 0.15;
        if self.position.x - attack.position.x < 0.0 {
            angle = PI - angle;
        }
        let knockback =
            (attack.knockback - self.knockback_resist).max(0.0);
        self.velocity = Vec2::from_angle(angle).scale(knockback);

        let splatter_position = self.collider.unwrap().get_center();
        let splatter_velocity = Vec2::from_angle(angle).scale(100.0);
        self.particles_emitter
            .init_blood_splatter(splatter_position, splatter_velocity);
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

    pub fn force_start_healing(&mut self) {
        self.healing.as_mut().unwrap().timer.force_start();
        self.particles_emitter
            .init_healing(self.collider.unwrap().get_center());
    }

    pub fn force_stop_healing(&mut self) {
        self.healing.as_mut().unwrap().timer.force_stop();
        self.particles_emitter.init_empty();
    }

    pub fn force_start_jumping(&mut self) {
        let jumping = self.jumping.as_mut().unwrap();
        jumping.timer.force_start();
        if let Some(stamina) = self.stamina.as_mut() {
            stamina.sub(jumping.stamina_cost);
        }
    }

    pub fn force_start_dashing(&mut self) {
        let dashing = self.dashing.as_mut().unwrap();
        dashing.timer.force_start();
        if let Some(stamina) = self.stamina.as_mut() {
            stamina.sub(dashing.stamina_cost);
        }
    }

    pub fn force_attack(&mut self) {
        use Orientation::*;

        let weapon = &mut self.weapons[self.weapon_idx];
        let collider = weapon.get_collider(self.orientation);
        weapon.timer.force_start();
        if let Some(stamina) = self.stamina.as_mut() {
            stamina.sub(weapon.stamina_cost);
        }
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
        self.weapons[self.weapon_idx].timer.check_if_started()
    }

    pub fn check_if_jumping(&self) -> bool {
        if let Some(jumping) = self.jumping {
            jumping.timer.check_if_started()
        } else {
            false
        }
    }

    pub fn check_if_dashing(&self) -> bool {
        if let Some(dashing) = self.dashing {
            dashing.timer.check_if_started()
        } else {
            false
        }
    }

    pub fn check_if_healing(&self) -> bool {
        if let Some(healing) = self.healing {
            healing.timer.check_if_started()
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

    pub fn check_if_enough_stamina_for_jumping(&self) -> bool {
        if let (Some(jumping), Some(stamina)) =
            (self.dashing, self.stamina)
        {
            if stamina.current < jumping.stamina_cost {
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

    pub fn check_if_jumping_ready(&self) -> bool {
        if let Some(jumping) = self.jumping {
            return jumping.timer.check_if_ready() && self.is_on_floor;
        }

        false
    }

    pub fn check_if_weapon_ready(&self) -> bool {
        self.weapons[self.weapon_idx].timer.check_if_ready()
    }

    pub fn check_if_dashing_ready(&self) -> bool {
        if let Some(dashing) = self.dashing {
            self.dashing.as_ref().unwrap().timer.check_if_ready()
        } else {
            false
        }
    }

    pub fn check_if_healing_ready(&self) -> bool {
        if let Some(healing) = self.healing {
            self.healing.as_ref().unwrap().timer.check_if_ready()
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
        self.is_on_stair = false;
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
                    Collider::Stair(rect) => {
                        if rect.collide_with_rect(self_rect)
                            && (rect.get_center().x
                                - self_rect.get_center().x)
                                .abs()
                                < 0.5 * rect.get_size().x
                        {
                            self.is_on_stair = true;
                        }
                    }
                };
            }
        }
    }

    fn update_weapons(&mut self, dt: f32, attacks: &mut Vec<Attack>) {
        let is_player = self.check_if_player();
        for weapon in self.weapons.iter_mut() {
            if let Some(attack) = weapon.update(
                dt,
                self.position,
                self.orientation,
                is_player,
            ) {
                attacks.push(attack);
            }
        }
    }

    fn update_jumping(&mut self, dt: f32) {
        use Orientation::*;

        if let Some(jumping) = self.jumping.as_mut() {
            self.velocity += jumping.update(dt, self.orientation);
        }
    }

    fn update_dashing(&mut self, dt: f32) {
        use Orientation::*;

        if let Some(dashing) = self.dashing.as_mut() {
            self.position.x += dashing.update(dt, self.orientation);
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

    pub fn update_particles_emitter(&mut self, dt: f32) {
        self.particles_emitter.update(dt, self.position);
    }

    pub fn update(
        &mut self,
        gravity: f32,
        friction: f32,
        colliders: &Vec<Collider>,
        attacks: &mut Vec<Attack>,
        dt: f32,
    ) {
        self.time += dt;

        if self.state != State::Dead {
            self.update_weapons(dt, attacks);
            self.update_jumping(dt);
            self.update_dashing(dt);
            self.update_healing(dt);
            self.update_stamina(dt);
        }

        self.update_kinematic(gravity, friction, colliders, dt);
        self.update_animator(dt);
        self.update_particles_emitter(dt);
    }

    pub fn play_animation(&mut self, name: &'static str) {
        self.animator.as_mut().unwrap().play(name);
    }

    pub fn pause_animation(&mut self) {
        self.animator.as_mut().unwrap().pause();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AbilityState {
    Ready,
    Anticipation,
    Action,
    Recovery,
    Cooldown,
}

#[derive(Clone, Copy, Debug)]
pub struct AbilityTimer {
    state: AbilityState,
    state_time: f32,
    anticipation_time: f32,
    action_time: f32,
    recovery_time: f32,
    cooldown_time: f32,
}

impl AbilityTimer {
    pub fn new(
        anticipation_time: f32,
        action_time: f32,
        recovery_time: f32,
        cooldown_time: f32,
    ) -> Self {
        Self {
            state: AbilityState::Ready,
            state_time: 0.0,
            anticipation_time,
            action_time,
            recovery_time,
            cooldown_time,
        }
    }

    pub fn check_if_ready(&self) -> bool {
        self.state == AbilityState::Ready
    }

    pub fn check_if_anticipation(&self) -> bool {
        self.state == AbilityState::Anticipation
    }

    pub fn check_if_action(&self) -> bool {
        self.state == AbilityState::Action
    }

    pub fn check_if_recovery(&self) -> bool {
        self.state == AbilityState::Recovery
    }

    pub fn check_if_cooldown(&self) -> bool {
        self.state == AbilityState::Cooldown
    }

    pub fn check_if_started(&self) -> bool {
        use AbilityState::*;
        self.state == Anticipation
            || self.state == Action
            || self.state == Recovery
    }

    pub fn force_start(&mut self) {
        self.state = AbilityState::Anticipation;
        self.state_time = 0.0;
    }

    pub fn force_stop(&mut self) {
        if self.state == AbilityState::Action {
            self.state = AbilityState::Recovery;
            self.state_time = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        use AbilityState::*;

        let mut is_action_started = false;
        self.state_time += dt;
        match self.state {
            Anticipation => {
                if self.state_time >= self.anticipation_time {
                    self.state_time = 0.0;
                    self.state = AbilityState::Action;
                    is_action_started = true;
                }
            }
            Action => {
                if self.state_time >= self.action_time {
                    self.state_time = 0.0;
                    self.state = AbilityState::Recovery;
                }
            }
            Recovery => {
                if self.state_time >= self.recovery_time {
                    self.state_time = 0.0;
                    self.state = AbilityState::Cooldown;
                }
            }
            Cooldown => {
                if self.state_time >= self.cooldown_time {
                    self.state_time = 0.0;
                    self.state = AbilityState::Ready;
                }
            }
            Ready => {}
        }

        is_action_started
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Jumping {
    pub speed: f32,
    pub stamina_cost: f32,
    pub angle: f32,
    pub timer: AbilityTimer,
}

impl Jumping {
    pub fn new(
        speed: f32,
        stamina_cost: f32,
        angle: f32,
        timer: AbilityTimer,
    ) -> Self {
        Self {
            speed,
            stamina_cost,
            angle,
            timer,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        orientation: Orientation,
    ) -> Vec2<f32> {
        use Orientation::*;

        let is_action_started = self.timer.update(dt);
        if is_action_started {
            let step = Vec2::new(self.angle.cos(), self.angle.sin())
                .scale(self.speed);
            return match orientation {
                Left => step.mul_x(-1.0),
                Right => step,
            };
        }

        Vec2::zeros()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dashing {
    pub speed: f32,
    pub stamina_cost: f32,
    pub timer: AbilityTimer,
}

impl Dashing {
    pub fn new(
        speed: f32,
        stamina_cost: f32,
        timer: AbilityTimer,
    ) -> Self {
        Self {
            speed,
            stamina_cost,
            timer,
        }
    }

    pub fn update(&mut self, dt: f32, orientation: Orientation) -> f32 {
        use Orientation::*;

        self.timer.update(dt);
        if self.timer.state == AbilityState::Action {
            let step = self.speed * dt;
            return match orientation {
                Left => -step,
                Right => step,
            };
        }

        0.0
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
    timer: AbilityTimer,
}

impl Healing {
    pub fn new(speed: f32, timer: AbilityTimer) -> Self {
        Self { speed, timer }
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        self.timer.update(dt);
        if self.timer.state == AbilityState::Action {
            return self.speed * dt;
        }

        0.0
    }
}

#[derive(Clone, Copy)]
pub struct Weapon {
    collider: Rect,
    pub damage: f32,
    pub knockback: f32,
    pub stamina_cost: f32,
    pub timer: AbilityTimer,
}

impl Weapon {
    pub fn new(
        collider: Rect,
        damage: f32,
        knockback: f32,
        stamina_cost: f32,
        timer: AbilityTimer,
    ) -> Self {
        Self {
            collider,
            damage,
            knockback,
            stamina_cost,
            timer,
        }
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

    pub fn update(
        &mut self,
        dt: f32,
        position: Vec2<f32>,
        orientation: Orientation,
        is_player: bool,
    ) -> Option<Attack> {
        let is_action_started = self.timer.update(dt);

        if is_action_started {
            let attack = Attack::new(
                position,
                self.get_collider(orientation),
                self.damage,
                self.knockback,
                self.timer.anticipation_time,
                is_player,
            );

            return Some(attack);
        }

        None
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Attack {
    pub position: Vec2<f32>,
    collider: Rect,
    pub damage: f32,
    pub knockback: f32,
    pub delay: f32,
    pub is_player_friendly: bool,
}

impl Attack {
    pub fn new(
        position: Vec2<f32>,
        collider: Rect,
        damage: f32,
        knockback: f32,
        delay: f32,
        is_player_friendly: bool,
    ) -> Self {
        Self {
            position,
            collider,
            damage,
            knockback,
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
    is_paused: bool,
}

impl Animator {
    pub fn new(default_sprite: AnimatedSprite) -> Self {
        let mut animation_to_sprite = HashMap::new();
        animation_to_sprite.insert("default", default_sprite);

        Self {
            flip: false,
            animation: "default",
            animation_to_sprite,
            is_paused: false,
        }
    }

    pub fn get_current_cycle(&self) -> f32 {
        self.animation_to_sprite.get(self.animation).unwrap().cycle
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
        self.is_paused = false;
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
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
        if !self.is_paused {
            self.animation_to_sprite
                .get_mut(self.animation)
                .unwrap()
                .update(dt);
        }
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

#[derive(Copy, Clone)]
struct Particle {
    pub ttl: f32,
    pub position: Vec2<f32>,
    pub size: f32,
    pub velocity: Vec2<f32>,
    pub acceleration: Vec2<f32>,
    pub fade_rate: f32,
    pub color: Color,
}

impl Particle {
    pub fn init(
        &mut self,
        ttl: f32,
        position: Vec2<f32>,
        size: f32,
        velocity: Vec2<f32>,
        acceleration: Vec2<f32>,
        fade_rate: f32,
        color: Color,
    ) {
        self.ttl = ttl;
        self.position = position;
        self.size = size;
        self.velocity = velocity;
        self.acceleration = acceleration;
        self.fade_rate = fade_rate;
        self.color = color;
    }

    pub fn empty() -> Self {
        Self {
            ttl: 0.0,
            position: Vec2::zeros(),
            size: 0.0,
            velocity: Vec2::zeros(),
            acceleration: Vec2::zeros(),
            fade_rate: 0.0,
            color: Color::red(0.0),
        }
    }

    pub fn check_if_alive(&self) -> bool {
        self.ttl > 0.0
    }

    pub fn update(&mut self, dt: f32) {
        if self.check_if_alive() {
            self.ttl -= dt;
            self.velocity += self.acceleration.scale(dt);
            self.position += self.velocity.scale(dt);
            self.color.a = (self.color.a - self.fade_rate * dt).max(0.0);
        }
    }

    pub fn get_draw_primitive(
        &self,
        position: Vec2<f32>,
    ) -> DrawPrimitive {
        let rect = Rect::from_center(
            self.position + position,
            Vec2::new(self.size, self.size),
        );

        DrawPrimitive::from_rect(
            rect,
            SpaceType::WorldSpace,
            0.0,
            0,
            self.color,
        )
    }
}

const MAX_N_PARTICLES: usize = 10;

#[derive(Clone)]
pub struct ParticlesEmitter {
    time: f32,
    ttl: f32,
    position: Vec2<f32>,

    emit_period: f32,
    n_to_emit: i32,
    n_emit_per_step_range: (usize, usize),

    particle_position_range: (f32, f32),
    particle_color: Color,
    particle_fade_rate: f32,
    particle_velocity: Vec2<f32>,
    particle_acceleration: Vec2<f32>,
    particle_size: f32,
    particle_ttl: f32,

    particles: [Particle; MAX_N_PARTICLES],
    first_particle_idx: usize,
    n_particles: usize,
}

impl ParticlesEmitter {
    pub fn empty() -> Self {
        let particles = [Particle::empty(); MAX_N_PARTICLES];
        Self {
            time: 0.0,
            ttl: 0.0,
            position: Vec2::zeros(),
            emit_period: 1.0,
            n_to_emit: 0,
            n_emit_per_step_range: (0, 0),
            particle_position_range: (0.0, 0.0),
            particle_color: Color::red(0.0),
            particle_fade_rate: 0.0,
            particle_velocity: Vec2::zeros(),
            particle_acceleration: Vec2::zeros(),
            particle_size: 1.0,
            particle_ttl: 0.0,
            particles,
            first_particle_idx: 0,
            n_particles: 0,
        }
    }

    pub fn init_empty(&mut self) {
        *self = Self::empty();
    }

    pub fn init_torch(&mut self, position: Vec2<f32>) {
        self.ttl = f32::INFINITY;
        self.position = position;
        self.emit_period = 0.25;
        self.n_to_emit = -1;
        self.n_emit_per_step_range = (1, 3);
        self.particle_position_range = (4.0, 8.0);
        self.particle_color = Color::red(1.0);
        self.particle_fade_rate = 2.0;
        self.particle_velocity = Vec2::new(0.0, 50.0);
        self.particle_acceleration = Vec2::new(0.0, 0.0);
        self.particle_size = 2.0;
        self.particle_ttl = 0.2;
    }

    pub fn init_blood_splatter(
        &mut self,
        position: Vec2<f32>,
        velocity: Vec2<f32>,
    ) {
        self.ttl = 1.0;
        self.position = position;
        self.emit_period = 0.0;
        self.n_to_emit = 8;
        self.n_emit_per_step_range = (8, 8);
        self.particle_position_range = (6.0, 6.0);
        self.particle_color = Color::red(1.0);
        self.particle_fade_rate = 1.0;
        self.particle_velocity = velocity;
        self.particle_acceleration = Vec2::new(0.0, -250.0);
        self.particle_size = 2.0;
        self.particle_ttl = 0.5;
    }

    pub fn init_healing(&mut self, position: Vec2<f32>) {
        self.init_torch(position);
        self.particle_color = Color::green(1.0);
    }

    pub fn torch(position: Vec2<f32>) -> Self {
        let mut emitter = Self::empty();
        emitter.init_torch(position);

        emitter
    }

    pub fn blood_splatter(
        position: Vec2<f32>,
        direction: Vec2<f32>,
    ) -> Self {
        let mut emitter = Self::empty();
        emitter.init_blood_splatter(position, direction);

        emitter
    }

    pub fn check_if_alive(&self) -> bool {
        self.ttl > 0.0 && (self.n_to_emit == -1 || self.n_to_emit > 0)
    }

    pub fn update(&mut self, dt: f32, position: Vec2<f32>) {
        self.ttl -= dt;
        self.time += dt;

        let n_emit = if !self.check_if_alive() {
            0
        } else if self.emit_period < f32::EPSILON {
            if self.n_to_emit != -1 {
                self.n_to_emit
            } else {
                panic!("Can't emit particles when the emit_period = 0 and n_to_emit == -1")
            }
        } else {
            let n_steps = (self.time / self.emit_period) as usize;
            self.time -= n_steps as f32 * self.emit_period;
            let mut n_emit = 0;

            for _ in 0..n_steps {
                n_emit += urand(
                    self.n_emit_per_step_range.0,
                    self.n_emit_per_step_range.1,
                ) as i32;

                if self.n_to_emit != -1 && n_emit > self.n_to_emit {
                    n_emit = self.n_to_emit;
                    break;
                }
            }

            n_emit
        };

        if self.n_to_emit != -1 {
            self.n_to_emit -= n_emit;
        }

        for _ in 0..n_emit {
            let idx = if self.n_particles == 0 {
                0
            } else {
                (self.first_particle_idx + self.n_particles)
                    % MAX_N_PARTICLES
            };

            self.n_particles = self.n_particles + 1;

            if self.n_particles > MAX_N_PARTICLES {
                self.n_particles = MAX_N_PARTICLES;
                self.first_particle_idx =
                    (self.first_particle_idx + 1) % MAX_N_PARTICLES;
            }

            let particle_position =
                Vec2::frand(self.particle_position_range);
            self.particles[idx].init(
                self.particle_ttl,
                particle_position,
                self.particle_size,
                self.particle_velocity,
                self.particle_acceleration,
                self.particle_fade_rate,
                self.particle_color,
            );
        }

        for i in 0..self.n_particles {
            let idx = (self.first_particle_idx + i) % MAX_N_PARTICLES;
            self.particles[idx].update(dt);
        }
    }

    pub fn get_draw_primitives(
        &self,
        position: Vec2<f32>,
    ) -> Vec<DrawPrimitive> {
        let mut primitives = Vec::with_capacity(self.n_particles);

        for i in 0..self.n_particles {
            let idx = (self.first_particle_idx + i) % MAX_N_PARTICLES;
            let particle = self.particles[idx];
            if particle.check_if_alive() {
                let primitive =
                    particle.get_draw_primitive(self.position + position);
                primitives.push(primitive);
            }
        }

        primitives
    }
}
