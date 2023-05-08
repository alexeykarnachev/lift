#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::entity::*;
use crate::graphics::*;
use crate::input::Input;
use crate::prefabs::*;
use crate::vec::*;

#[derive(PartialEq)]
pub enum WorldState {
    Play,
    GameOver,
}

pub struct World {
    pub state: WorldState,

    pub camera: Camera,
    pub shaft: Entity,
    pub lift: Entity,
    pub player: Entity,
    pub enemies: Vec<Vec<Entity>>,

    pub floors: Vec<Entity>,

    pub game_over: Entity,

    pub sprite_atlas: SpriteAtlas,
    pub glyph_atlas: GlyphAtlas,
}

impl World {
    pub fn new(n_floors: usize) -> Self {
        let sprite_atlas = create_default_sprite_atlas();
        let glyph_atlas = create_default_glyph_atlas();
        let mut floors = Vec::with_capacity(n_floors as usize);
        let mut enemies = Vec::with_capacity(n_floors as usize);
        for floor_idx in 0..n_floors {
            let floor = create_floor_entity(floor_idx);
            let floor_y = floor.position.y;
            floors.push(floor);

            let n_enemies = 4;
            let mut floor_enemies = Vec::with_capacity(n_enemies);
            for enemy_idx in 0..n_enemies {
                let side = if enemy_idx % 2 == 1 { -1.0 } else { 1.0 };
                let x = (2.0 + 2.0 * enemy_idx as f32) * side;
                let position = Vec2::new(x, floor_y);

                let mut knight =
                    create_knight_entity(position, &sprite_atlas);
                knight.text = Some(Text::from_glyph_atlas(
                    &glyph_atlas,
                    Space::World,
                    Origin::Center(Vec2::zeros()),
                    "Knight! Go, go, go...".to_string(),
                    Color::new(1.0, 1.0, 0.0, 1.0),
                    0.005,
                ));

                floor_enemies.push(knight);
            }

            enemies.push(floor_enemies);
        }

        let idx = (n_floors / 2) as usize;
        let mut lift = create_lift_entity(idx);
        lift.text = Some(Text::from_glyph_atlas(
            &glyph_atlas,
            Space::World,
            Origin::Center(Vec2::zeros()),
            "LIFT!".to_string(),
            Color::new(1.0, 0.0, 1.0, 1.0),
            0.015,
        ));
        let shaft = create_shaft_entity(n_floors);

        let position = Vec2::new(0.0, lift.position.y);
        let mut player = create_knight_entity(position, &sprite_atlas);
        player.kinematic = None;

        let state = WorldState::Play;
        let camera = Camera::new(Vec2::new(0.0, lift.position.y));

        let mut game_over = Entity::new(Vec2::zeros());
        game_over.text = Some(Text::from_glyph_atlas(
            &glyph_atlas,
            Space::Screen,
            Origin::Center(Vec2::new(0.0, 62.0)),
            "Game Over".to_string(),
            Color::new(1.0, 0.0, 0.0, 1.0),
            2.0,
        ));

        Self {
            state,
            camera,
            shaft,
            lift,
            player,
            enemies,
            floors,
            game_over,
            sprite_atlas,
            glyph_atlas,
        }
    }

    pub fn update(&mut self, dt: f32, input: &Input) {
        use WorldState::*;
        match self.state {
            Play => {
                self.update_lift(dt, input);
                self.update_floors();
                self.update_enemies(dt);
                self.update_player(dt);
                self.update_free_camera(input);
                if self.player.is_dead() {
                    self.state = WorldState::GameOver;
                }
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

        let mut target = None;
        if let Some(floor_idx) = self.get_lift_floor_idx() {
            let shaft_width = self.shaft.get_draw_primitive_size().x;
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
    }

    pub fn update_floors(&mut self) {
        let lift_floor_idx = self.get_lift_floor_idx_f();
        for (idx, floor) in self.floors.iter_mut().enumerate() {
            let gray = 0.5
                - (0.6 * (idx as f32 - lift_floor_idx).abs()).powf(2.0);
            floor.set_draw_primitive_color(Color::new_gray(gray, 1.0));
        }
    }

    pub fn update_enemies(&mut self, dt: f32) {
        let floor_idx = if let Some(idx) = self.get_lift_floor_idx() {
            idx
        } else {
            return;
        };

        for enemy in self.enemies[floor_idx].iter_mut() {
            attack(enemy, &mut self.player, dt);
            enemy.update_animator(dt);
        }
    }

    pub fn update_player(&mut self, dt: f32) {
        self.player.position.y = self.lift.position.y;

        let floor_idx = if let Some(idx) = self.get_lift_floor_idx() {
            idx
        } else {
            return;
        };

        let collider = self.player.get_collider();
        let mut nearest_enemy = None;
        let mut nearest_dist = f32::INFINITY;
        for (idx, enemy) in self.enemies[floor_idx].iter_mut().enumerate()
        {
            let dist = collider.get_x_dist_to(enemy.position.x);
            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_enemy = Some(enemy);
            }
        }

        if let Some(enemy) = nearest_enemy {
            attack(&mut self.player, enemy, dt);
            self.player.update_animator(dt);
        }
    }

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

    pub fn get_floor_height(&self) -> f32 {
        self.floors[0].get_draw_primitive_size().y
    }

    pub fn get_lift_floor_idx_f(&self) -> f32 {
        self.lift.position.y / self.get_floor_height()
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

        (floor.position.y / self.get_floor_height()).floor() as usize
    }

    pub fn get_lift_floor(&self) -> Option<&Entity> {
        if let Some(idx) = self.get_lift_floor_idx() {
            return Some(&self.floors[idx]);
        }

        None
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
            position,
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

fn attack(entity: &mut Entity, target: &mut Entity, dt: f32) {
    let target_position = &target.position;
    let target_collider = target.collider.as_ref().unwrap();
    let target_width = target_collider.get_width();
    let target_health = target.health.as_mut().unwrap();

    let mut position = &mut entity.position;
    let weapon = entity.weapon.as_mut().unwrap();
    let collider = entity.collider.as_ref().unwrap();
    let width = collider.get_width();
    let animator = entity.animator.as_mut().unwrap();

    let dist = target_position.x - position.x;
    let attack_dist = weapon.range + 0.5 * (target_width + width);
    let can_reach = dist.abs() < attack_dist;
    let mut damage = 0.0;
    if let (Some(kinematic), false) =
        (entity.kinematic.as_mut(), can_reach)
    {
        kinematic.speed = kinematic.max_speed * dist.signum();
        let step = dt * kinematic.speed;
        position.x += step;
        animator.play("run");
    } else if weapon.is_ready() && can_reach {
        weapon.cooldown = 0.0;
        damage += weapon.damage;
        animator.play("attack");
    }

    weapon.cooldown += dt;
    animator.flip = dist < 0.0;
    target_health.current -= damage;
}
