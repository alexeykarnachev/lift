#![allow(dead_code)]
#![allow(unused_variables)]

use crate::input::Input;
use crate::vec::Vec2;
use image::imageops::flip_vertical_in_place;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(PartialEq)]
pub enum WorldState {
    Play,
    GameOver,
}

pub struct World {
    pub state: WorldState,

    pub camera: Camera,
    pub lift: Lift,
    pub player: Player,
    pub enemies: Vec<Vec<Enemy>>,

    pub shaft_width: f32,

    pub floors: Vec<Floor>,
    pub floor_size: Vec2<f32>,

    pub sprite_atlas: SpriteAtlas,
}

impl World {
    pub fn new(n_floors: usize, floor_size: Vec2<f32>) -> Self {
        let sprite_atlas = SpriteAtlas::new(
            "./assets/sprites/atlas.json",
            "./assets/sprites/atlas.png",
        );

        let mut floors = Vec::with_capacity(n_floors as usize);
        let mut enemies = Vec::with_capacity(n_floors as usize);
        for floor_idx in 0..n_floors {
            let y = floor_idx as f32 * floor_size.y;
            let floor = Floor {
                size: floor_size,
                y,
                idx: floor_idx,
            };
            floors.push(floor);

            let n_enemies = 4;
            let mut floor_enemies = Vec::with_capacity(n_enemies);
            for enemy_idx in 0..n_enemies {
                let side = if enemy_idx % 2 == 1 { -1.0 } else { 1.0 };
                let position =
                    Vec2::new(2.0 + 2.0 * enemy_idx as f32, 0.0)
                        .scale(side);
                let weapon = Weapon {
                    range: 0.2,
                    speed: 1.0,
                    damage: 100.0,
                    cooldown: 0.0,
                };
                let mut animator =
                    Animator::new(AnimatedSprite::from_atlas(
                        &sprite_atlas,
                        "knight_idle",
                        2.0,
                    ));
                animator.add(
                    "idle",
                    AnimatedSprite::from_atlas(
                        &sprite_atlas,
                        "knight_idle",
                        2.0,
                    ),
                );
                animator.add(
                    "attack",
                    AnimatedSprite::from_atlas(
                        &sprite_atlas,
                        "knight_attack",
                        2.0,
                    ),
                );
                animator.add(
                    "run",
                    AnimatedSprite::from_atlas(
                        &sprite_atlas,
                        "knight_run",
                        2.0,
                    ),
                );

                let enemy = Enemy {
                    size: Vec2::new(0.5, 0.8),
                    position: position,
                    max_speed: 2.0,
                    weapon: weapon,
                    animator: animator,
                };
                floor_enemies.push(enemy);
            }

            enemies.push(floor_enemies);
        }

        let idx = (n_floors / 2) as usize;
        let max_speed = 4.0;
        let floor = &floors[idx];
        let lift = Lift::from_floor(floor, max_speed);
        let shaft_width = lift.size.x * 1.2;

        let player = Player {
            size: lift.size * Vec2::new(0.25, 0.4),
            position: Vec2::new(0.0, 0.0),
            max_health: 1000.0,
            health: 1000.0,
        };

        let state = WorldState::Play;
        let camera = Camera::new(Vec2::new(0.0, floor.y));

        Self {
            state,
            camera,
            lift,
            player,
            enemies,
            shaft_width,
            floors,
            floor_size,
            sprite_atlas,
        }
    }

    pub fn update(&mut self, dt: f32, input: &Input) {
        use WorldState::*;
        match self.state {
            Play => {
                self.update_lift(dt, input);
                self.update_enemies(dt);
                self.update_player(dt);
                self.update_free_camera(input);
            }
            GameOver => {}
        }
    }

    fn update_lift(&mut self, dt: f32, input: &Input) {
        let cursor_world_pos = window_to_world(
            &self.camera,
            input.window_size,
            input.cursor_pos,
        );

        if let Some(floor) = self.get_lift_floor() {
            let n_enemies = self.enemies[floor.idx].len();
            let shaft_width = self.get_shaft_world_rect().get_size().x;
            let is_enemy_in_lift = (0..n_enemies).any(|enemy_idx| {
                let rect = self.get_enemy_world_rect(floor.idx, enemy_idx);
                let x = rect.bot_left.x.abs().min(rect.top_right.x.abs());
                x <= 0.5 * shaft_width
            });

            if !is_enemy_in_lift {
                let mut idx = floor.idx as i32;
                if let Some(_) = input.lmb_press_pos {
                    idx += 1;
                } else if let Some(_) = input.rmb_press_pos {
                    idx -= 1;
                }

                idx = idx.clamp(0, self.floors.len() as i32 - 1);
                self.lift.target_y = idx as f32 * self.floor_size.y;
            }
        }

        let diff = self.lift.target_y - self.lift.y;
        let step = dt * self.lift.max_speed;
        if step >= diff.abs() {
            self.lift.y = self.lift.target_y;
        } else {
            self.lift.y += step * diff.signum();
        }
    }

    pub fn update_enemies(&mut self, dt: f32) {
        let floor_idx;
        if let Some(floor) = self.get_lift_floor() {
            floor_idx = floor.idx;
        } else {
            return;
        };

        let player_position = self.player.position;
        let player_width = self.player.size.x;
        let mut damage = 0.0;
        for enemy in self.enemies[floor_idx].iter_mut() {
            // let dist = player_position.x - enemy.position.x;
            // let attack_dist =
            //     enemy.weapon.range + 0.5 * (player_width + enemy.size.x);
            // if dist.abs() > attack_dist {
            //     let step = dt * enemy.max_speed * dist.signum();
            //     enemy.position.x += step;
            //     enemy.animator.play("run");
            // } else if enemy.weapon.cooldown >= 1.0 / enemy.weapon.speed {
            //     enemy.weapon.cooldown = 0.0;
            //     damage += enemy.weapon.damage;
            //     enemy.animator.play("attack");
            // } else {
            //     enemy.animator.play("idle");
            // }

            enemy.animator.play("attack");
            enemy.animator.update(dt);
            enemy.weapon.cooldown += dt;
        }

        self.player.health = (self.player.health - damage).max(0.0);
        if self.player.health <= 0.0 {
            self.state = WorldState::GameOver;
        }
    }

    pub fn update_player(&mut self, dt: f32) {}

    fn update_free_camera(&mut self, input: &Input) {
        self.camera.aspect =
            input.window_size.x as f32 / input.window_size.y as f32;
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

    pub fn get_lift_floor_idx(&self) -> f32 {
        self.lift.y / self.floor_size.y
    }

    pub fn get_lift_nearest_floor(&self) -> &Floor {
        let idx = self.get_lift_floor_idx().round() as usize;

        &self.floors[idx]
    }

    pub fn get_lift_floor(&self) -> Option<&Floor> {
        let idx = self.get_lift_floor_idx();
        if (idx.floor() - idx).abs() < 1.0e-5 {
            return Some(&self.floors[idx as usize]);
        }

        None
    }

    pub fn get_shaft_world_rect(&self) -> Rect {
        let height = self.floors.len() as f32 * self.floor_size.y;
        let size = Vec2::new(self.shaft_width, height);

        Rect {
            bot_left: Vec2::new(-size.x * 0.5, 0.0),
            top_right: Vec2::new(size.x * 0.5, size.y),
        }
    }

    pub fn get_floor_world_rect(&self, floor_idx: usize) -> Rect {
        let y = self.floor_size.y * (floor_idx as f32 + 0.5);
        let center = Vec2::new(0.0, y);
        let rect = Rect {
            bot_left: center - self.floor_size.scale(0.5),
            top_right: center + self.floor_size.scale(0.5),
        };

        Rect {
            bot_left: center - self.floor_size.scale(0.5),
            top_right: center + self.floor_size.scale(0.5),
        }
    }

    pub fn get_lift_world_rect(&self) -> Rect {
        let y = self.lift.y + 0.5 * self.lift.size.y;
        let center = Vec2::new(0.0, y);

        Rect {
            bot_left: center - self.lift.size.scale(0.5),
            top_right: center + self.lift.size.scale(0.5),
        }
    }

    pub fn get_player_world_rect(&self) -> Rect {
        let x = 0.0 + self.player.position.x;
        let local_y = self.player.position.y + 0.5 * self.player.size.y;
        let y = self.lift.y + local_y;
        let center = Vec2::new(x, y);

        Rect {
            bot_left: center - self.player.size.scale(0.5),
            top_right: center + self.player.size.scale(0.5),
        }
    }

    pub fn get_enemy_world_rect(
        &self,
        floor_idx: usize,
        enemy_idx: usize,
    ) -> Rect {
        let enemy = &self.enemies[floor_idx][enemy_idx];
        let x = 0.0 + enemy.position.x;
        let local_y = enemy.position.y + 0.5 * enemy.size.y;
        let y = self.floors[floor_idx].y + local_y;
        let center = Vec2::new(x, y);

        Rect {
            bot_left: center - enemy.size.scale(0.5),
            top_right: center + enemy.size.scale(0.5),
        }
    }

    pub fn get_enemy_sprite(
        &self,
        floor_idx: usize,
        enemy_idx: usize,
    ) -> Sprite {
        let enemy = &self.enemies[floor_idx][enemy_idx];

        *enemy.animator.get_sprite()
    }
}

#[derive(Copy, Clone)]
pub struct Rect {
    pub bot_left: Vec2<f32>,
    pub top_right: Vec2<f32>,
}

impl Rect {
    pub fn from_center(center: Vec2<f32>, size: Vec2<f32>) -> Self {
        let half_size = size.scale(0.5);

        Self {
            bot_left: center - half_size,
            top_right: center + half_size,
        }
    }

    pub fn get_center(&self) -> Vec2<f32> {
        (self.top_right + self.bot_left).scale(0.5)
    }

    pub fn get_size(&self) -> Vec2<f32> {
        self.top_right - self.bot_left
    }

    pub fn to_xywh(&self) -> [f32; 4] {
        let center = self.get_center();
        let size = self.get_size();

        [center.x, center.y, size.x, size.y]
    }
}

pub struct Camera {
    pub position: Vec2<f32>,
    pub orientation: f32,

    pub view_width: f32,
    pub aspect: f32,
}

impl Camera {
    fn new(position: Vec2<f32>) -> Self {
        Self {
            position: position,
            orientation: 0.0,
            view_width: 20.0,
            aspect: 1.77,
        }
    }

    pub fn get_view_size(&self) -> Vec2<f32> {
        let view_height = self.view_width / self.aspect;

        Vec2::new(self.view_width, view_height)
    }
}

pub struct Lift {
    pub size: Vec2<f32>,
    pub y: f32,
    pub target_y: f32,
    pub max_speed: f32,
    speed: f32,
}

impl Lift {
    pub fn new(size: Vec2<f32>, y: f32, max_speed: f32) -> Self {
        Self {
            size: size,
            y: y,
            target_y: y,
            max_speed: max_speed,
            speed: 0.0,
        }
    }

    pub fn from_floor(floor: &Floor, max_speed: f32) -> Self {
        let lift_size = Vec2::new(floor.size.y * 0.6, floor.size.y);

        Lift::new(lift_size, floor.y, max_speed)
    }
}

pub struct Player {
    pub size: Vec2<f32>,
    pub position: Vec2<f32>,

    pub max_health: f32,
    pub health: f32,
}

pub struct Enemy {
    pub size: Vec2<f32>,
    pub position: Vec2<f32>,

    pub max_speed: f32,
    pub weapon: Weapon,

    pub animator: Animator,
}

pub struct Weapon {
    pub range: f32,
    pub speed: f32,
    pub damage: f32,
    pub cooldown: f32,
}

pub struct Floor {
    pub size: Vec2<f32>,
    pub y: f32,

    pub idx: usize,
}

#[derive(Deserialize, Copy, Clone)]
pub struct Sprite {
    pub u: f32,
    pub v: f32,
    pub w: f32,
    pub h: f32,
    pub x_scale: f32,
    pub y_scale: f32,
}

impl Sprite {
    pub fn to_uvwh(&self) -> [f32; 4] {
        [self.u, self.v, self.w, self.h]
    }
}

#[derive(Deserialize)]
pub struct SpriteAtlas {
    pub file_name: String,
    pub size: [u32; 2],
    pub sprites: HashMap<String, Vec<Sprite>>,

    #[serde(skip)]
    pub image: DynamicImage,
}

impl SpriteAtlas {
    pub fn new(meta_file_path: &str, image_file_path: &str) -> Self {
        let meta = fs::read_to_string(meta_file_path).unwrap();
        let mut atlas: Self = serde_json::from_str(&meta).unwrap();

        let mut image = ImageReader::open(image_file_path)
            .unwrap()
            .decode()
            .unwrap();
        flip_vertical_in_place(&mut image);

        atlas.image = image;

        atlas
    }
}

pub struct AnimatedSprite {
    pub name: &'static str,
    pub duration: f32,
    current_duration: f32,

    frames: Vec<Sprite>,
}

impl AnimatedSprite {
    pub fn from_atlas(
        atlas: &SpriteAtlas,
        name: &'static str,
        duration: f32,
    ) -> Self {
        let frames = atlas.sprites.get(name).unwrap_or_else(|| {
            panic!("There is no such sprite in the sprite atlas: {}", name)
        });

        Self {
            name,
            duration,
            current_duration: 0.0,
            frames: frames.to_vec(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.current_duration += dt;
    }

    pub fn get_current_frame(&self) -> &Sprite {
        let mut cycle = self.current_duration / self.duration;
        cycle -= cycle.floor();
        let frame_idx = (cycle * self.frames.len() as f32).floor();

        &self.frames[frame_idx as usize]
    }
}

pub struct Animator {
    current_animation: &'static str,
    animation_to_sprite: HashMap<&'static str, AnimatedSprite>,
}

impl Animator {
    pub fn new(default_sprite: AnimatedSprite) -> Self {
        let mut animation_to_sprite = HashMap::new();
        animation_to_sprite.insert("default", default_sprite);

        Self {
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

    pub fn get_sprite(&self) -> &Sprite {
        self.animation_to_sprite
            .get(self.current_animation)
            .unwrap()
            .get_current_frame()
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_to_sprite
            .get_mut(self.current_animation)
            .unwrap()
            .update(dt);
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
    let mut world_pos = bot_left + view_size * window_pos / window_size;
    world_pos = world_pos.rotate(Vec2::zeros(), camera.orientation);
    return world_pos;
}

pub fn world_to_screen(
    camera: &Camera,
    window_size: Vec2<i32>,
    world_pos: Vec2<f32>,
) -> Vec2<i32> {
    let view_size = camera.get_view_size();
    let bot_left = camera.position - view_size.scale(0.5);
    let view_pos = world_pos.rotate(camera.position, -camera.orientation)
        - camera.position;
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
