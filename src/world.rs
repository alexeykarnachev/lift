#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]

use crate::entity::*;
use crate::graphics::*;
use crate::input::*;
use crate::prefabs::*;
use crate::ui::*;
use crate::vec::*;
use std::f32::consts::PI;
use std::fs;

#[derive(PartialEq, Debug)]
pub enum WorldState {
    Play,
    GameOver,
    Restart,
    Quit,
}

pub struct World {
    pub state: WorldState,

    pub time: f32,
    pub gravity: f32,

    pub camera: Camera,
    pub shaft: Shaft,
    pub lift: Lift,
    pub player: Entity,
    pub enemies: Vec<Vec<Entity>>,
    pub spawners: Vec<Vec<Spawner>>,
    pub melee_attacks: Vec<MeleeAttack>,
    pub bullets: Vec<Bullet>,

    pub floors: Vec<Floor>,

    pub play_ui: UI,
    pub game_over_ui: UI,
    pub game_over_ui_last_modified: u64,

    pub sprite_atlas: SpriteAtlas,
    pub glyph_atlas: GlyphAtlas,
}

impl World {
    pub fn new() -> Self {
        let n_floors = 16;
        let sprite_atlas = create_default_sprite_atlas();
        let glyph_atlas = create_default_glyph_atlas();

        let mut floors = Vec::with_capacity(n_floors);
        let mut enemies = Vec::with_capacity(n_floors);
        let mut spawners = Vec::with_capacity(n_floors);
        for floor_idx in 0..n_floors {
            let floor = create_floor(floor_idx);
            let floor_enemies = Vec::with_capacity(128);
            let mut floor_spawners = Vec::with_capacity(1024);

            // floor_spawners
            //     .push(create_rat_spawner(Vec2::new(-8.0, floor.y), &sprite_atlas));
            floor_spawners.push(create_rat_spawner(
                Vec2::new(8.0, floor.y),
                &sprite_atlas,
            ));

            enemies.push(floor_enemies);
            spawners.push(floor_spawners);
            floors.push(floor);
        }

        let melee_attacks: Vec<MeleeAttack> = Vec::with_capacity(256);
        let bullets: Vec<Bullet> = Vec::with_capacity(256);

        let idx = (n_floors / 2) as usize;
        let mut lift = create_lift_entity(idx);
        let shaft = create_shaft(n_floors);

        let position = Vec2::new(0.0, lift.y);
        let mut player = create_player(position);

        let camera = Camera::new(Vec2::new(0.0, lift.y));

        let play_ui = create_default_play_ui();
        let mut game_over_ui = create_default_game_over_ui();
        let game_over_ui_last_modified =
            get_last_modified(game_over_ui.file_path);

        Self {
            state: WorldState::Play,
            time: 0.0,
            gravity: 9.8,
            camera,
            shaft,
            lift,
            player,
            enemies,
            spawners,
            melee_attacks,
            bullets,
            floors,
            play_ui,
            game_over_ui,
            game_over_ui_last_modified,
            sprite_atlas,
            glyph_atlas,
        }
    }

    pub fn update(&mut self, dt: f32, input: &Input) {
        self.hot_reload();

        self.camera.aspect =
            input.window_size.x as f32 / input.window_size.y as f32;

        use WorldState::*;
        match self.state {
            Play => {
                self.update_play_ui(input);
                self.update_bullets(dt);
                self.update_melee_attacks(dt);
                self.update_enemies(dt);
                self.update_player(dt, input);
                self.update_free_camera(input);
                self.update_lift(dt, input);
                self.update_spawners(dt);
                self.time += dt;
            }
            GameOver => {
                self.update_game_over_ui(input);
            }
            Restart => {
                *self = Self::new();
            }
            Quit => {}
        }

        use Flag::*;
        if self.player.check_flag(Dead) && self.state == Play {
            self.state = GameOver;
        }
    }

    fn hot_reload(&mut self) {
        let game_over_ui_last_modified =
            get_last_modified(self.game_over_ui.file_path);
        if game_over_ui_last_modified != self.game_over_ui_last_modified {
            self.game_over_ui = create_default_game_over_ui();
        }
    }

    fn update_lift(&mut self, dt: f32, input: &Input) {
        let cursor_world_pos = window_to_world(
            &self.camera,
            input.window_size,
            input.cursor_pos,
        );

        let mut target = None;
        if let Some(floor_idx) = self.get_lift_floor_idx() {
            let shaft_width = self.shaft.get_collider().get_size().x;
            let is_enemy_in_lift =
                self.enemies[floor_idx].iter().any(|enemy| {
                    let collider = enemy.get_collider();
                    let x = collider.get_x_dist_to(0.0);

                    x <= 0.5 * shaft_width
                });

            if !is_enemy_in_lift {
                let mut idx = floor_idx as i32;
                if let Some(_) = input.lmb_press_pos {
                    idx += 1;
                } else if let Some(_) = input.rmb_press_pos {
                    idx -= 1;
                }

                idx = idx.clamp(0, self.floors.len() as i32 - 1);
                let target_y = idx as f32 * self.get_floor_height();
                target = Some(Vec2::new(0.0, target_y));
            }
        }

        /*
        let kinematic = self.lift.kinematic.as_mut().unwrap();
        if let Some(target) = target {
            kinematic.target = Some(target);
        }

        let position = &mut self.lift.position;
        if let Some(target) = kinematic.target {
            let diff = target.y - position.y;
            let step = dt * kinematic.max_speed;
            if step >= diff.abs() {
                position.y = target.y;
            } else {
                position.y += step * diff.signum();
            }
        }
        */
    }

    pub fn update_spawners(&mut self, dt: f32) {
        let floor_idx = if let Some(idx) = self.get_lift_floor_idx() {
            idx
        } else {
            return;
        };

        let floor_spawners = &mut self.spawners[floor_idx];
        let floor_enemies = &mut self.enemies[floor_idx];
        for spawner in floor_spawners.iter_mut() {
            if let Some(entity) = spawner.update(dt) {
                floor_enemies.push(entity);
            }
        }
    }

    pub fn update_enemies(&mut self, dt: f32) {
        use AnimationType::*;
        use Behaviour::*;
        use Flag::*;

        let floor_idx = if let Some(idx) = self.get_lift_floor_idx() {
            idx
        } else {
            return;
        };

        let enemies = &mut self.enemies[floor_idx];
        let floor_collider = self.floors[floor_idx].get_collider();
        let floor_y = floor_collider.get_y_min();
        let target = self.player.get_center();
        let player_collider = self.player.get_collider();
        let is_player_alive = !self.player.check_flag(Dead);

        for enemy in enemies.iter_mut().filter(|e| !e.check_flag(Dead)) {
            if !is_player_alive {
                continue;
            }

            let position = enemy.position;
            let mut animation =
                enemy.animator.as_ref().unwrap().animation_type;

            match enemy.behaviour {
                Rat => {
                    if enemy.check_if_can_reach_by_melee(
                        player_collider,
                        self.time,
                    ) {
                        let attack = enemy.attack_by_melee(self.time);
                        self.melee_attacks.push(attack);
                        // animation = Attack;
                    } else if enemy.check_if_can_jump(floor_y, self.time)
                        && position.dist_to(target) <= 2.0
                    {
                        let mut angle = PI * 0.1;
                        if target.x - position.x < 0.0 {
                            angle = PI - angle;
                        }
                        enemy.jump_at_angle(angle, self.time);
                        animation = Jump;
                    } else if enemy.check_if_can_step(floor_y, self.time) {
                        enemy.immediate_step(target.x - position.x, dt);
                        animation = Move;
                    } else if enemy.check_if_on_floor(floor_y) {
                        animation = Idle;
                    }

                    let is_on_floor = enemy.check_if_on_floor(floor_y);
                    let animator = enemy.animator.as_mut().unwrap();
                    animator.play(animation);
                    if is_on_floor {
                        animator.flip = target.x > position.x;
                    }
                }
                _ => {
                    panic!(
                        "Enemy behaviour: {:?} is not implemented",
                        enemy.behaviour
                    )
                }
            }

            enemy.update_kinematic(self.gravity, floor_collider, dt);
            enemy.update_animator(dt);
        }
    }

    pub fn update_player(&mut self, dt: f32, input: &Input) {
        let floor_collider = if let Some(floor) = self.get_lift_floor() {
            floor.get_collider()
        } else {
            self.lift.get_collider()
        };

        let floor_y = floor_collider.get_y_min();
        let position = &mut self.player.position;

        use Keyaction::*;
        if input.is_action(Right) {
            self.player.immediate_step(1.0, dt)
        }
        if input.is_action(Left) {
            self.player.immediate_step(-1.0, dt)
        }

        if input.is_action(Up)
            && self.player.check_if_can_jump(floor_y, self.time)
        {
            self.player.jump_at_angle(PI * 0.5, self.time);
        }

        self.player
            .update_kinematic(self.gravity, floor_collider, dt);

        // Shoot
        if input.lmb_is_down {
            let target = window_to_world(
                &self.camera,
                input.window_size,
                input.cursor_pos,
            );
            if self.player.check_if_can_reach_by_range(target, self.time) {
                let bullet =
                    self.player.attack_by_range(target, self.time);
                self.bullets.push(bullet);
            }
        }
    }

    pub fn update_bullets(&mut self, dt: f32) {
        let floor_idx = if let Some(idx) = self.get_lift_floor_idx() {
            idx
        } else {
            return;
        };

        let floor = &self.floors[floor_idx];
        let floor_enemies = &mut self.enemies[floor_idx];

        let floor_collider = floor.get_collider();
        let mut new_bullets = Vec::with_capacity(self.bullets.len());

        use Flag::*;
        'bullet: for bullet in self.bullets.iter_mut() {
            let step = bullet.velocity.scale(dt);
            bullet.position += step;
            if floor_collider.collide_with_point(bullet.position) {
                if bullet.is_player_friendly {
                    for enemy in floor_enemies
                        .iter_mut()
                        .filter(|e| !e.check_flag(Dead))
                    {
                        if enemy.try_receive_bullet_damage(bullet) {
                            if enemy.check_flag(Dead) {
                                self.player.score += 100;
                            }
                            continue 'bullet;
                        }
                    }
                } else if !self.player.check_flag(Dead) {
                    if self.player.try_receive_bullet_damage(bullet) {
                        continue 'bullet;
                    }
                }

                new_bullets.push(bullet.clone());
            }
        }

        self.bullets = new_bullets;
    }

    pub fn update_melee_attacks(&mut self, dt: f32) {
        let floor_idx = if let Some(idx) = self.get_lift_floor_idx() {
            idx
        } else {
            return;
        };

        let floor = &self.floors[floor_idx];
        let floor_enemies = &mut self.enemies[floor_idx];

        let floor_collider = floor.get_collider();

        use Flag::*;
        'attack: for attack in self.melee_attacks.iter() {
            if attack.is_player_friendly {
                for enemy in floor_enemies
                    .iter_mut()
                    .filter(|e| !e.check_flag(Dead))
                {
                    if enemy.try_receive_melee_attack_damage(attack) {
                        if enemy.check_flag(Dead) {
                            self.player.score += 100;
                        }
                        continue 'attack;
                    }
                }
            } else if !self.player.check_flag(Dead) {
                if self.player.try_receive_melee_attack_damage(attack) {
                    continue 'attack;
                }
            }
        }

        self.melee_attacks.clear();
    }

    fn update_free_camera(&mut self, input: &Input) {
        if input.wheel_d != 0 {
            let diff = self.camera.view_width * 0.1 * input.wheel_d as f32;
            self.camera.view_width -= diff;
        }

        if input.mmb_is_down {
            let cursor_world_pos = window_to_world(
                &self.camera,
                input.window_size,
                input.cursor_pos,
            );
            let cursor_world_prev_pos = window_to_world(
                &self.camera,
                input.window_size,
                input.cursor_prev_pos,
            );
            let cursor_world_diff =
                cursor_world_pos - cursor_world_prev_pos;
            self.camera.position -= cursor_world_diff;
        }
    }

    fn update_play_ui(&mut self, input: &Input) {
        let score = format!("Score: {}", self.player.score);
        self.play_ui.set_element_text("score", &score);
        _ = self.play_ui.update(input, &self.glyph_atlas);
    }

    fn update_game_over_ui(&mut self, input: &Input) {
        if let Some(event) =
            self.game_over_ui.update(input, &self.glyph_atlas)
        {
            match event {
                UIEvent::LMBPress(id) => match id.as_str() {
                    "restart" => self.state = WorldState::Restart,
                    "quit" => {
                        self.state = WorldState::Quit;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn get_floor_height(&self) -> f32 {
        self.floors[0].get_collider().get_height()
    }

    pub fn get_lift_floor_idx_f(&self) -> f32 {
        self.lift.y / self.get_floor_height()
    }

    pub fn get_lift_floor_idx(&self) -> Option<usize> {
        let idx = self.get_lift_floor_idx_f();
        if (idx.floor() - idx).abs() < 1.0e-5 {
            return Some(idx as usize);
        }

        None
    }

    pub fn get_lift_nearest_floor_idx(&self) -> usize {
        let idx = self.get_lift_floor_idx_f().round() as usize;
        let floor = &self.floors[idx];

        (floor.y / self.get_floor_height()).floor() as usize
    }

    pub fn get_lift_floor(&self) -> Option<&Floor> {
        if let Some(idx) = self.get_lift_floor_idx() {
            return Some(&self.floors[idx]);
        }

        None
    }
}

pub struct Camera {
    pub position: Vec2<f32>,

    pub view_width: f32,
    pub aspect: f32,
}

impl Camera {
    fn new(position: Vec2<f32>) -> Self {
        Self {
            position,
            view_width: 20.0,
            aspect: 1.77,
        }
    }

    pub fn get_view_size(&self) -> Vec2<f32> {
        let view_height = self.view_width / self.aspect;

        Vec2::new(self.view_width, view_height)
    }
}

pub fn window_to_world(
    camera: &Camera,
    window_size: Vec2<i32>,
    window_pos: Vec2<i32>,
) -> Vec2<f32> {
    let width = window_size.x as f32;
    let height = window_size.y as f32;
    let window_size = Vec2::new(width, height);

    let view_size = camera.get_view_size();

    let window_pos = Vec2::<f32>::new(
        window_pos.x as f32,
        height - window_pos.y as f32,
    );
    let bot_left = camera.position - view_size.scale(0.5);
    let world_pos = bot_left + view_size * window_pos / window_size;
    return world_pos;
}

pub fn world_to_screen(
    camera: &Camera,
    window_size: Vec2<i32>,
    world_pos: Vec2<f32>,
) -> Vec2<i32> {
    let view_size = camera.get_view_size();
    let bot_left = camera.position - view_size.scale(0.5);
    let view_pos = world_pos - camera.position;
    let ndc_pos = view_pos.scale(2.0) / view_size;
    let window_size =
        Vec2::new(window_size.x as f32, window_size.y as f32);
    let window_pos =
        window_size.scale(0.5) * (ndc_pos + Vec2::new(1.0, 1.0));
    let window_pos = Vec2::new(
        window_pos.x as i32,
        (window_size.y - window_pos.y) as i32,
    );

    window_pos
}

fn get_last_modified(file_path: &str) -> u64 {
    let metadata = fs::metadata(file_path).unwrap();
    metadata.modified().unwrap().elapsed().unwrap().as_millis() as u64
}
