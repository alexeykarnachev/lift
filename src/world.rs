#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]

use crate::entity::*;
use crate::graphics::*;
use crate::input::*;
use crate::level::*;
use crate::prefabs::*;
use crate::ui::*;
use crate::vec::*;
use std::f32::consts::PI;
use std::fs;
use std::time::SystemTime;

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
    pub friction: f32,

    pub level: Level,

    pub camera: Camera,
    pub melee_attacks: Vec<MeleeAttack>,
    pub bullets: Vec<Bullet>,

    pub play_ui: UI,
    pub game_over_ui: UI,
    pub game_over_ui_last_modified: u64,

    pub sprite_atlas: SpriteAtlas,
    pub glyph_atlas: GlyphAtlas,
}

impl World {
    pub fn new() -> Self {
        let sprite_atlas = create_default_sprite_atlas();
        let glyph_atlas = create_default_glyph_atlas();
        let level = Level::new("./assets/levels/0.json", &sprite_atlas);

        let melee_attacks: Vec<MeleeAttack> = Vec::with_capacity(256);
        let bullets: Vec<Bullet> = Vec::with_capacity(256);
        let camera = Camera::new(level.player.get_center().add_y(2.0));

        let play_ui = create_default_play_ui();
        let mut game_over_ui = create_default_game_over_ui();
        let game_over_ui_last_modified =
            get_last_modified(game_over_ui.file_path);

        Self {
            state: WorldState::Play,
            time: 0.0,
            gravity: 400.0,
            friction: 0.02,
            level,
            camera,
            melee_attacks,
            bullets,
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
                self.update_lights(dt);
                self.update_free_camera(input);
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

        if self.level.player.check_if_dead() && self.state == Play {
            self.state = GameOver;
        }
    }

    fn hot_reload(&mut self) {
        let game_over_ui_last_modified =
            get_last_modified(self.game_over_ui.file_path);
        if game_over_ui_last_modified != self.game_over_ui_last_modified {
            self.game_over_ui_last_modified = game_over_ui_last_modified;
            self.game_over_ui = create_default_game_over_ui();
        }
    }

    pub fn update_spawners(&mut self, dt: f32) {
        // let floor_spawners = &mut self.spawners[floor_idx];
        // let floor_enemies = &mut self.enemies[floor_idx];
        // for spawner in floor_spawners.iter_mut() {
        //     if let Some(entity) = spawner.update(dt) {
        //         floor_enemies.push(entity);
        //     }
        // }
    }

    pub fn update_enemies(&mut self, dt: f32) {
        use Behaviour::*;

        let enemies = &mut self.level.enemies;
        let player = &self.level.player;
        let room = self.level.room;
        let floor_y = room.get_y_min();
        let ceil_y = room.get_y_max();
        let target = player.get_center();
        let player_collider = player.get_collider().unwrap();
        let is_player_alive = !player.check_if_dead();

        for enemy in enemies.iter_mut() {
            if !is_player_alive {
                continue;
            }

            let position = enemy.get_center();
            let dist_to_target = position.dist_to(target);

            match enemy.behaviour {
                Some(Rat {
                    min_jump_distance,
                    max_jump_distance,
                }) => {
                    if enemy.check_if_dead() {
                        enemy.play_animation("death");
                    } else if enemy
                        .check_if_can_reach_by_melee(player_collider)
                    {
                        let attack = if enemy.check_if_on_floor(floor_y) {
                            enemy.play_animation("melee_attack");
                            enemy.attack_by_melee(None)
                        } else {
                            enemy.attack_by_melee(Some(0.0))
                        };
                        self.melee_attacks.push(attack);
                    } else if enemy.check_if_can_jump(floor_y)
                        && dist_to_target <= max_jump_distance
                        && dist_to_target >= min_jump_distance
                    {
                        let mut angle = PI * 0.1;
                        if target.x - position.x < 0.0 {
                            angle = PI - angle;
                        }
                        enemy.jump_at_angle(angle, None);
                        enemy.play_animation("jump");
                    } else if enemy.check_if_can_step(floor_y) {
                        let direction = (target - position).with_y(0.0);
                        enemy.immediate_step(direction, dt);
                        enemy.play_animation("move");
                    } else if enemy.check_if_on_floor(floor_y)
                        && enemy.check_if_cooling_down()
                    {
                        enemy.play_animation("idle");
                    }

                    let can_flip = enemy.check_if_on_floor(floor_y)
                        && !enemy.check_if_dead();
                    if can_flip {
                        let is_flip = target.x > position.x;
                        enemy.animator.as_mut().unwrap().flip = is_flip;
                        enemy.set_orientation(is_flip);
                    }
                }
                Some(Bat) => {
                    let deviation =
                        Vec2::from_angle(self.time * 4.0).scale(0.5);
                    let t = enemy.get_time_since_last_received_damage();
                    if t < 0.3 {
                        enemy.apply_gravity = true;
                    } else if !enemy.check_if_dead() {
                        enemy.apply_gravity = false;
                        enemy.velocity.y = enemy.velocity.y.max(0.0);
                    }

                    if enemy.check_if_dead() {
                        enemy.apply_gravity = true;
                        enemy.play_animation("death");
                    } else if enemy.get_health_ratio() < 0.6
                        && enemy.check_if_can_start_healing()
                    {
                        if enemy.check_if_on_ceil(ceil_y) {
                            enemy.force_start_healing();
                            enemy.velocity.x = 0.0;
                        } else {
                            let direction =
                                Vec2::new(0.0, 1.0) + deviation;
                            enemy.immediate_step(direction, dt);
                            enemy.play_animation("wave");
                        }
                    } else if enemy
                        .check_if_can_reach_by_melee(player_collider)
                        && !enemy.check_if_healing()
                    {
                        let attack = enemy.attack_by_melee(None);
                        self.melee_attacks.push(attack);
                        enemy.play_animation("melee_attack");
                    } else if enemy.check_if_can_step(floor_y)
                        && !enemy.check_if_healing()
                    {
                        let direction =
                            (target - position).norm() + deviation;
                        enemy.immediate_step(direction, dt);
                        enemy.play_animation("wave");
                    } else if !enemy.check_if_healing()
                        && enemy.check_if_cooling_down()
                    {
                        enemy.immediate_step(deviation, dt * 0.3);
                        enemy.play_animation("wave");
                    } else if enemy.check_if_healing() {
                        enemy.play_animation("sleep");
                    }

                    let can_flip = !enemy.check_if_dead()
                        && !enemy.check_if_healing();
                    let animator = enemy.animator.as_mut().unwrap();
                    if can_flip {
                        let is_flip = target.x > position.x;
                        animator.flip = is_flip;
                        enemy.set_orientation(is_flip);
                    }
                }
                _ => {
                    panic!(
                        "Enemy behaviour: {:?} is not implemented",
                        enemy.behaviour
                    )
                }
            }

            enemy.update(self.gravity, self.friction, room, dt);
        }
    }

    pub fn update_player(&mut self, dt: f32, input: &Input) {
        let room = self.level.room;
        let floor_y = room.get_y_min();
        let player = &mut self.level.player;
        let position = player.position;
        let is_attacking =
            player.check_if_attacking() || player.check_if_cooling_down();
        let is_dashing = player.check_if_dashing();

        use Keyaction::*;
        if let (Some(_), false, false) =
            (input.lmb_press_pos, is_attacking, is_dashing)
        {
            let attack = player.attack_by_melee(None);
            self.melee_attacks.push(attack);
            player.animator.as_mut().unwrap().reset();
            player.animator.as_mut().unwrap().play("attack");
        } else if input.is_action(Right)
            && player.check_if_can_step(floor_y)
        {
            if input.is_action(Down) && player.check_if_can_start_dashing()
            {
                player.force_start_dashing();
                player.animator.as_mut().unwrap().play("slide");
            } else {
                player.immediate_step(Vec2::new(1.0, 0.0), dt);
                player.animator.as_mut().unwrap().play("run");
            }
            player.animator.as_mut().unwrap().flip = false;
            player.set_orientation(true);
        } else if input.is_action(Left)
            && player.check_if_can_step(floor_y)
        {
            if input.is_action(Down) && player.check_if_can_start_dashing()
            {
                player.force_start_dashing();
                player.animator.as_mut().unwrap().play("slide");
            } else {
                player.immediate_step(Vec2::new(-1.0, 0.0), dt);
                player.animator.as_mut().unwrap().play("run");
            }
            player.animator.as_mut().unwrap().flip = true;
            player.set_orientation(false);
        } else if !is_attacking && !is_dashing {
            player.animator.as_mut().unwrap().play("idle");
        }

        player.update(self.gravity, self.friction, room, dt);
    }

    pub fn update_bullets(&mut self, dt: f32) {
        let room = self.level.room;
        let enemies = &mut self.level.enemies;
        let player = &mut self.level.player;
        let mut new_bullets = Vec::with_capacity(self.bullets.len());

        'bullet: for bullet in self.bullets.iter_mut() {
            let step = bullet.velocity.scale(dt);
            bullet.position += step;
            if room.collide_with_point(bullet.position) {
                if bullet.is_player_friendly {
                    for enemy in
                        enemies.iter_mut().filter(|e| !e.check_if_dead())
                    {
                        if enemy.try_receive_bullet_damage(bullet) {
                            if enemy.check_if_dead() {
                                player.score += 100;
                            }
                            continue 'bullet;
                        }
                    }
                } else if !player.check_if_dead() {
                    if player.try_receive_bullet_damage(bullet) {
                        continue 'bullet;
                    }
                }

                new_bullets.push(bullet.clone());
            }
        }

        self.bullets = new_bullets;
    }

    pub fn update_melee_attacks(&mut self, dt: f32) {
        let room = self.level.room;
        let enemies = &mut self.level.enemies;
        let player = &mut self.level.player;
        let mut new_melee_attacks =
            Vec::with_capacity(self.melee_attacks.len());

        'attack: for attack in self.melee_attacks.iter_mut() {
            attack.delay -= dt;
            if attack.delay > 0.0 {
                new_melee_attacks.push(attack.clone());
                continue 'attack;
            }

            if attack.is_player_friendly {
                for enemy in
                    enemies.iter_mut().filter(|e| !e.check_if_dead())
                {
                    if enemy.try_receive_melee_attack_damage(attack) {
                        let mut angle = PI * 0.15;
                        if enemy.position.x - player.position.x < 0.0 {
                            angle = PI - angle;
                        }
                        enemy.jump_at_angle(angle, Some(120.0));

                        if enemy.check_if_dead() {
                            player.score += 100;
                        }
                    }
                }
            } else if !player.check_if_dead() {
                if player.try_receive_melee_attack_damage(attack) {
                    continue 'attack;
                }
            }
        }

        self.melee_attacks = new_melee_attacks;
    }

    fn update_lights(&mut self, dt: f32) {
        for light in self.level.lights.iter_mut() {
            light.update(self.gravity, self.friction, self.level.room, dt)
        }
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
        let score = format!("Score: {}", self.level.player.score);
        let health_ratio = self.level.player.get_health_ratio();
        self.play_ui.set_element_string("score", &score);
        self.play_ui
            .set_element_color("health", Color::healthbar(health_ratio));
        self.play_ui.set_element_filling("health", health_ratio);

        _ = self.play_ui.update(input, &self.glyph_atlas);
    }

    fn update_game_over_ui(&mut self, input: &Input) {
        use UIEvent::*;

        let events = self.game_over_ui.update(input, &self.glyph_atlas);
        for event in events.iter() {
            match event {
                LMBPress(id) => match id.as_str() {
                    "restart" => self.state = WorldState::Restart,
                    "quit" => {
                        self.state = WorldState::Quit;
                    }
                    _ => {}
                },
                Hover(id) => {
                    self.game_over_ui.set_element_color(
                        &id,
                        Color::new(0.9, 0.9, 0.5, 1.0),
                    );
                }
                Empty(id) => {
                    self.game_over_ui
                        .set_element_color(&id, Color::default());
                }
                _ => {}
            }
        }
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
            view_width: 500.0,
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
    if let Ok(time) = metadata.modified() {
        time.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    } else {
        0
    }
}
