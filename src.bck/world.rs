#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]

use crate::entity::*;
use crate::graphics::*;
use crate::gui::*;
use crate::input::*;
use crate::level::*;
use crate::player_stats::*;
use crate::prefabs::*;
use crate::utils::smooth_step;
use crate::vec::*;
use std::f32::consts::PI;
use std::fs;
use std::time::SystemTime;

#[derive(PartialEq, Debug)]
pub enum WorldState {
    MainMenu,
    Play,
    GameOver,
    SkillsTree,
    Quit,
}

pub struct World {
    pub state: WorldState,

    pub time: f32,
    pub gravity: f32,
    pub friction: f32,

    pub level: Level,

    pub camera: Camera,
    pub attacks: Vec<Attack>,

    pub gui: GUI,

    pub sprite_atlas: SpriteAtlas,
    pub glyph_atlas: GlyphAtlas,
}

impl World {
    pub fn new() -> Self {
        let sprite_atlas = create_default_sprite_atlas();
        let glyph_atlas = create_default_glyph_atlas();
        let level = Level::new("./assets/levels/0.json", &sprite_atlas);

        let attacks: Vec<Attack> = Vec::with_capacity(256);
        let camera = Camera::new(level.player.get_center().add_y(64.0));

        let gui = GUI::new();

        Self {
            // state: WorldState::MainMenu,
            state: WorldState::Play,
            time: 0.0,
            gravity: 400.0,
            friction: 0.02,
            level,
            camera,
            attacks,
            gui,
            sprite_atlas,
            glyph_atlas,
        }
    }

    pub fn update(&mut self, mut dt: f32, input: &mut Input) {
        self.camera.aspect =
            input.window_size.x as f32 / input.window_size.y as f32;

        use WorldState::*;
        match self.state {
            MainMenu => {
                self.update_main_menu_gui(input);
            }
            Play => {
                if input.take_action(Keyaction::SkillsTree) {
                    self.state = SkillsTree;
                } else {
                    self.update_game_gui(input);
                    self.update_attacks(dt);
                    self.update_player(dt, input);
                    self.update_enemies(dt);
                    self.update_lights(dt);
                    // self.update_free_camera(input);
                    self.update_player_camera(input);
                    self.time += dt;
                }
            }
            GameOver => {
                *self = Self::new();
            }
            SkillsTree => {
                if input.take_action(Keyaction::SkillsTree) {
                    self.state = Play;
                } else {
                    self.update_skills_tree_gui(input);
                }
            }
            Quit => {}
        }

        if self.level.player.check_if_dead() && self.state == Play {
            self.state = GameOver;
        }
    }

    pub fn update_enemies(&mut self, dt: f32) {
        use Behaviour::*;
        use State::*;

        let mut enemies_to_spawn = vec![];
        let enemies = &mut self.level.enemies;
        let player = &self.level.player;

        for enemy_id in 0..enemies.len() {
            let enemy = &mut enemies[enemy_id];
            let position = enemy.position;
            let behaviour = enemy.behaviour.unwrap();
            if enemy.state != Dead && enemy.check_if_dead() {
                enemy.state = Dead;
                if let Some(spawner_id) = enemy.spawner_id {
                    let spawner =
                        &mut enemies[spawner_id].spawner.as_mut().unwrap();
                    spawner.n_alive_current -= 1;
                }

                match behaviour {
                    RatNest => {
                        let rat_king =
                            create_rat_king(position, &self.sprite_atlas);
                        enemies_to_spawn.push(rat_king);
                    }
                    _ => {}
                }
            }

            let enemy = &mut enemies[enemy_id];
            let enemy_collider = enemy.get_collider().unwrap();
            let player_collider = player.get_collider().unwrap();
            let enemy_center = enemy.get_center();
            let player_center = player.get_center();
            let to_player = player_center - enemy_center;
            let to_player_orientation = to_player.x.signum() > 0.0;
            let dist_to_player = to_player.abs();

            let mut can_see_player =
                to_player.len() <= enemy.view_distance;
            if can_see_player && enemy.state != Dead {
                'collider: for collider in self.level.colliders.iter() {
                    match collider {
                        Collider::Rigid(rect) => {
                            if rect.collide_with_line(
                                enemy_center,
                                player_center,
                            ) {
                                can_see_player = false;
                                break 'collider;
                            }
                        }
                        _ => {}
                    }
                }
            }

            match enemy.behaviour.as_ref() {
                Some(Rat) => match enemy.state {
                    Initial => {
                        enemy.state = Idle;
                    }
                    Idle => {
                        enemy.play_animation("idle");
                        enemy.set_orientation(to_player_orientation);
                        enemy.set_weapon(0);

                        if can_see_player {
                            if enemy.check_if_can_reach_by_weapon(
                                player_collider,
                            ) {
                                enemy.force_attack();
                                enemy.state = Attacking;
                            } else {
                                enemy.state = Walking;
                            }
                        }
                    }
                    Walking => {
                        enemy.play_animation("move");
                        enemy.set_orientation(to_player_orientation);
                        enemy.set_weapon(0);

                        if !enemy.is_on_floor {
                            enemy.state = Falling;
                        } else if !can_see_player {
                            enemy.state = Idle;
                        } else if enemy.check_if_jumping_ready()
                            && dist_to_player.x >= 20.0
                            && dist_to_player.x <= 100.0
                        {
                            let mut angle = PI * 0.1;
                            if !to_player_orientation {
                                angle = PI - angle;
                            };
                            enemy.force_start_jumping();
                            enemy.state = Jumping;
                        } else if enemy
                            .check_if_can_reach_by_weapon(player_collider)
                        {
                            enemy.force_attack();
                            enemy.state = Attacking;
                        } else if to_player_orientation {
                            enemy.immediate_step(Vec2::right(), dt)
                        } else {
                            enemy.immediate_step(Vec2::left(), dt)
                        }
                    }
                    Attacking => {
                        enemy.play_animation("melee_attack");
                        if enemy.check_if_weapon_ready() {
                            enemy.state = Idle;
                        }
                    }
                    Jumping => {
                        enemy.play_animation("jump");
                        enemy.set_weapon(1);

                        let is_jumping = enemy.check_if_jumping();
                        if !is_jumping {
                            enemy.state = Idle;
                        } else if enemy_collider
                            .collide_with_rect(player_collider)
                            && enemy.check_if_weapon_ready()
                        {
                            enemy.force_attack();
                        }
                    }
                    Falling => {
                        enemy.play_animation("idle");
                        if enemy.is_on_floor {
                            enemy.state = Idle;
                        }
                    }
                    Dead => {
                        enemy.play_animation("death");
                    }
                    _ => {
                        panic!(
                            "Can't handle {:?} state for a Rat",
                            enemy.state
                        )
                    }
                },
                Some(Bat) => {
                    let deviation =
                        Vec2::from_angle(self.time * 4.0).scale(0.25);
                    let t = enemy.get_time_since_last_received_damage();
                    if t < 0.3 {
                        enemy.apply_gravity = true;
                    } else if !enemy.check_if_dead() {
                        enemy.apply_gravity = false;
                        enemy.velocity.y = 0.0;
                    }

                    match enemy.state {
                        Initial => {
                            enemy.state = Idle;
                        }
                        Idle => {
                            enemy.play_animation("wave");
                            enemy.set_orientation(to_player_orientation);

                            if can_see_player {
                                if enemy.check_if_can_reach_by_weapon(
                                    player_collider,
                                ) {
                                    enemy.force_attack();
                                    enemy.state = Attacking;
                                } else {
                                    enemy.state = Walking;
                                }
                            }
                        }
                        Walking => {
                            enemy.play_animation("wave");
                            enemy.set_orientation(to_player_orientation);

                            if enemy.get_health_ratio() < 0.6
                                && enemy.check_if_healing_ready()
                            {
                                if enemy.is_on_ceil {
                                    enemy.force_start_healing();
                                    enemy.velocity = Vec2::zeros();
                                    enemy.state = Healing;
                                } else {
                                    enemy.immediate_step(Vec2::up(), dt);
                                }
                            } else if !can_see_player {
                                enemy.state = Idle;
                            } else if enemy.check_if_can_reach_by_weapon(
                                player_collider,
                            ) {
                                enemy.force_attack();
                                enemy.state = Attacking;
                            } else {
                                enemy.immediate_step(
                                    to_player.norm() + deviation,
                                    dt,
                                )
                            }
                        }
                        Attacking => {
                            enemy.play_animation("melee_attack");
                            if enemy.check_if_weapon_ready() {
                                enemy.state = Idle;
                            }
                        }
                        Healing => {
                            enemy.play_animation("sleep");
                            if !enemy.check_if_healing()
                                || enemy.velocity.x != 0.0
                            {
                                enemy.force_stop_healing();
                                enemy.state = Idle;
                            }
                        }
                        Dead => {
                            enemy.play_animation("death");
                            enemy.apply_gravity = true;
                        }
                        _ => {
                            panic!(
                                "Can't handle {:?} state for a Bat",
                                enemy.state
                            )
                        }
                    }
                }
                Some(RatNest) => {
                    match enemy.state {
                        Initial => {
                            enemy.state = Idle;
                        }
                        Idle => {
                            enemy.play_animation("idle");
                        }
                        Dead => {
                            enemy.play_animation("death");
                        }
                        _ => {
                            panic!(
                                "Can't handle {:?} state for a RatNest",
                                enemy.state
                            )
                        }
                    }

                    if enemy.state != Dead {
                        if let Some(mut spawned) =
                            enemy.update_spawner(dt, &self.sprite_atlas)
                        {
                            enemies_to_spawn.push(spawned);
                        }
                    }
                }
                Some(RatKing) => match enemy.state {
                    Initial => {
                        enemy.play_animation("rise");
                        enemy.set_orientation(to_player_orientation);
                        if enemy.get_current_animation_cycle() >= 1.0 {
                            enemy.state = Idle;
                        }
                    }
                    Idle => {
                        enemy.play_animation("idle");
                        enemy.set_orientation(to_player_orientation);
                        enemy.set_weapon(0);

                        if can_see_player {
                            if enemy.check_if_can_reach_by_weapon(
                                player_collider,
                            ) {
                                enemy.force_attack();
                                enemy.state = Attacking;
                            } else {
                                enemy.state = Walking;
                            }
                        }
                    }
                    Walking => {
                        enemy.play_animation("move");
                        enemy.set_orientation(to_player_orientation);

                        if !enemy.is_on_floor {
                            enemy.state = Falling;
                        } else if !can_see_player {
                            enemy.state = Idle;
                        } else if enemy.check_if_dashing_ready()
                            && dist_to_player.x >= 50.0
                            && dist_to_player.x <= 200.0
                        {
                            enemy.state = Dashing;
                            enemy.force_start_dashing();
                        } else if enemy
                            .check_if_can_reach_by_weapon(player_collider)
                        {
                            enemy.force_attack();
                            enemy.state = Attacking;
                        } else if to_player_orientation {
                            enemy.immediate_step(Vec2::right(), dt)
                        } else {
                            enemy.immediate_step(Vec2::left(), dt)
                        }
                    }
                    Dashing => {
                        enemy.play_animation("roll");
                        enemy.set_weapon(1);

                        if !enemy.check_if_dashing() {
                            enemy.state = Idle;
                        } else if enemy_collider
                            .collide_with_rect(player_collider)
                            && enemy.check_if_weapon_ready()
                        {
                            enemy.force_attack();
                        }
                    }
                    Attacking => {
                        enemy.play_animation("melee_attack");
                        if enemy.check_if_weapon_ready() {
                            enemy.state = Idle;
                        }
                    }
                    Dead => {
                        enemy.play_animation("death");
                    }
                    _ => {
                        panic!(
                            "Can't handle {:?} state for a RatKing",
                            enemy.state
                        )
                    }
                },
                _ => {}
            }

            enemy.update(
                self.gravity,
                self.friction,
                &self.level.colliders,
                &mut self.attacks,
                dt,
            );
        }

        for enemy in enemies_to_spawn {
            self.level.spawn_enemy(enemy);
        }
    }

    pub fn update_player(&mut self, dt: f32, input: &Input) {
        use Keyaction::*;
        use State::*;

        let player = &mut self.level.player;
        if !player.is_on_stair {
            player.apply_gravity = true;
        } else {
            player.apply_gravity = false;
            player.velocity.y = 0.0;
        }

        let is_attack_action = input.is_action(Attack);
        let is_left_action = input.is_action(Left);
        let is_right_action = input.is_action(Right);
        let is_up_action = input.is_action(Up);
        let is_down_action = input.is_action(Down);

        match player.state {
            Initial => {
                player.state = Idle;
            }
            Idle => {
                player.play_animation("idle");
                if is_left_action || is_right_action {
                    player.state = Walking;
                } else if is_attack_action
                    && player.check_if_weapon_ready()
                    && player.check_if_enough_stamina_for_attack()
                {
                    player.force_attack();
                    player.state = Attacking;
                } else if (is_down_action || is_up_action)
                    && player.is_on_stair
                {
                    player.state = Climbing;
                }
            }
            Walking => {
                if !player.is_on_floor {
                    player.state = Falling;
                } else if (is_down_action || is_up_action)
                    && player.is_on_stair
                {
                    player.state = Climbing;
                } else if is_attack_action
                    && player.check_if_weapon_ready()
                    && player.check_if_enough_stamina_for_attack()
                {
                    player.force_attack();
                    player.state = Attacking;
                } else {
                    if is_left_action || is_right_action {
                        player.set_orientation(is_right_action);
                        player.play_animation("walk");
                        if input.is_action(Dash)
                            && player.check_if_dashing_ready()
                            && player.check_if_enough_stamina_for_dashing()
                        {
                            player.state = Dashing;
                            player.force_start_dashing();
                        } else if is_right_action {
                            player.immediate_step(Vec2::right(), dt);
                        } else {
                            player.immediate_step(Vec2::left(), dt);
                        }
                    } else {
                        player.play_animation("idle");
                        player.state = Idle;
                    }
                }
            }
            Dashing => {
                player.play_animation("roll");
                if !player.check_if_dashing() {
                    player.state = Idle;
                } else if (is_down_action || is_up_action)
                    && player.is_on_stair
                {
                    player.state = Climbing;
                }
            }
            Attacking => {
                player.play_animation("attack");
                if player.check_if_weapon_ready() {
                    player.state = Idle;
                }
            }
            Climbing => {
                if player.is_on_floor {
                    player.play_animation("idle");
                } else {
                    player.play_animation("climb");
                    player.set_orientation(true);
                }

                if !player.is_on_stair {
                    player.state = Idle;
                } else if is_up_action {
                    player.immediate_step(Vec2::up(), dt);
                } else if is_down_action {
                    player.immediate_step(Vec2::down(), dt);
                } else if is_left_action {
                    player.immediate_step(Vec2::left(), dt);
                } else if is_right_action {
                    player.immediate_step(Vec2::right(), dt);
                } else {
                    player.pause_animation();
                }
            }
            Falling => {
                player.play_animation("idle");
                if player.is_on_floor {
                    player.state = Idle;
                } else if player.is_on_stair {
                    player.state = Climbing;
                } else if is_right_action {
                    player.immediate_step(Vec2::right(), dt);
                    player.set_orientation(true);
                } else if is_left_action {
                    player.immediate_step(Vec2::left(), dt);
                    player.set_orientation(false);
                }
            }
            _ => {
                panic!(
                    "Can't handle {:?} state for a Player",
                    player.state
                )
            }
        }

        if !player.check_if_light_alive() {
            player.light = Some(Light::player());
        }

        player.update(
            self.gravity,
            self.friction,
            &self.level.colliders,
            &mut self.attacks,
            dt,
        );
    }

    pub fn update_attacks(&mut self, dt: f32) {
        let enemies = &mut self.level.enemies;
        let player = &mut self.level.player;
        let mut new_attacks = Vec::with_capacity(self.attacks.len());

        'attack: for attack in self.attacks.iter_mut() {
            attack.delay -= dt;
            if attack.delay > 0.0 {
                new_attacks.push(attack.clone());
                continue 'attack;
            }

            if attack.is_player_friendly {
                let mut attacked_enemies =
                    Vec::with_capacity(enemies.len());

                for enemy in
                    enemies.iter_mut().filter(|e| !e.check_if_dead())
                {
                    if enemy.collide_with_attack(attack) {
                        attacked_enemies.push(enemy);
                    }
                }

                // Calculate splash damage
                let n_enemies = attacked_enemies.len() as f32;
                let penalty = player.splash_damage_penalty;
                let max = attack.damage;
                let min = attack.damage / n_enemies;
                let damage = penalty * min + (1.0 - penalty) * max;
                attack.damage = damage;

                for enemy in attacked_enemies.iter_mut() {
                    enemy.receive_attack(attack);
                    if enemy.check_if_dead() {
                        player.add_exp(enemy.exp_drop)
                    }
                }
            } else if !player.check_if_dead() {
                if player.collide_with_attack(attack) {
                    player.receive_attack(attack);
                    continue 'attack;
                }
            }
        }

        self.attacks = new_attacks;
    }

    fn update_lights(&mut self, dt: f32) {
        for light in self.level.lights.iter_mut() {
            light.update(
                self.gravity,
                self.friction,
                &self.level.colliders,
                &mut self.attacks,
                dt,
            )
        }
    }

    fn update_player_camera(&mut self, input: &Input) {
        let ws = input.window_size;
        let cp = self.camera.position;
        let pp = self.level.player.position.add_y(64.0);
        let sp = world_to_screen(&self.camera, ws, pp);
        let sp = Vec2::new(
            2.0 * sp.x as f32 / ws.x as f32 - 1.0,
            2.0 * sp.y as f32 / ws.y as f32 - 1.0,
        );
        let k = 0.2;
        if sp.x > k {
            self.camera.position.x += 0.25 * ws.x as f32 * (sp.x - k);
        } else if sp.x < -k {
            self.camera.position.x += 0.25 * ws.x as f32 * (sp.x + k);
        }

        if sp.y > k {
            self.camera.position.y -= 0.25 * ws.y as f32 * (sp.y - k);
        } else if sp.y < -k {
            self.camera.position.y -= 0.25 * ws.y as f32 * (sp.y + k);
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

    fn update_skills_tree_gui(&mut self, input: &mut Input) {
        use DrawDirection::*;
        use SkillsChainType::*;
        use WorldState::*;

        let stats = self.level.player.stats.as_mut().unwrap();

        self.gui.begin(input);

        self.gui.start_group();

        self.gui.set_cursor_at_top_left();
        self.gui.advance_cursor(50.0, -50.0);
        self.gui.set_draw_direction(Down);
        self.gui.text(
            &format!("Points: {:?}", stats.n_skill_points),
            &self.glyph_atlas,
        );

        let mut hovered = None;
        self.gui.advance_cursor(0.0, -80.0);
        let cursor = self.gui.get_ui_cursor();
        hovered = hovered.or(self.update_skills_chain_gui(Attack));
        self.gui.set_cursor_at(cursor.add_y(-100.0));
        hovered = hovered.or(self.update_skills_chain_gui(Agility));
        self.gui.set_cursor_at(cursor.add_y(-200.0));
        hovered = hovered.or(self.update_skills_chain_gui(Durability));
        self.gui.set_cursor_at(cursor.add_y(-300.0));
        hovered = hovered.or(self.update_skills_chain_gui(Light));

        self.gui.set_font_size(16);
        self.gui.set_cursor_at(cursor.add_y(-350.0));
        if let Some(skill) = hovered {
            self.gui.text(&skill.description, &self.glyph_atlas);
        } else {
            self.gui.text(" ", &self.glyph_atlas);
        }

        self.gui.group_background();
    }

    fn update_skills_chain_gui(
        &mut self,
        type_: SkillsChainType,
    ) -> Option<Skill> {
        use ButtonState::*;
        use DrawDirection::*;
        use SkillsChainType::*;

        let sprite_name = match type_ {
            Attack => "attack_skills",
            Durability => "durability_skills",
            Agility => "agility_skills",
            Light => "light_skills",
        };
        let arrow_sprite = self.sprite_atlas.sprites["skills_arrow"][0];
        let stats = self.level.player.stats.as_mut().unwrap();
        let has_points = stats.n_skill_points > 0;
        let skills = stats.get_skills_chain_by_type(type_);
        let n_learned = skills.n_learned;
        let n_skills = skills.skills.len();

        let mut hovered_skill = None;
        let mut learned_skill_type = None;
        self.gui.set_horizontal_padding(20.0);
        self.gui.set_draw_direction(Right);
        for i in 0..n_skills {
            let skill_sprite = self.sprite_atlas.sprites[sprite_name][i];
            let state = if i < n_learned {
                self.gui.sprite(skill_sprite, 1.0)
            } else if i == n_learned && has_points {
                self.gui.sprite_button(skill_sprite)
            } else {
                self.gui.sprite(skill_sprite, 0.1)
            };

            if i < n_skills - 1 {
                let alpha = if n_learned > 0 && i < n_learned - 1 {
                    1.0
                } else if i < n_learned {
                    0.4
                } else {
                    0.1
                };
                self.gui.sprite(arrow_sprite, alpha);
            }

            match state {
                Released | Active | Hot => {
                    hovered_skill = Some(skills.skills[i].clone());
                    if state == Released {
                        learned_skill_type = Some(type_);
                    }
                }
                _ => {}
            }
        }

        if let Some(type_) = learned_skill_type {
            self.level
                .player
                .stats
                .as_mut()
                .unwrap()
                .force_learn_next(type_);
        }

        hovered_skill
    }

    fn update_main_menu_gui(&mut self, input: &mut Input) {
        use ButtonState::*;
        use DrawDirection::*;
        use WorldState::*;

        self.gui.begin(input);

        self.gui.set_cursor_at_bot_left();
        self.gui.advance_cursor(25.0, 25.0);

        self.gui.set_font_size(37);
        self.gui.set_draw_direction(Up);
        if self.gui.text_button("Quit", &self.glyph_atlas) == Released {
            self.state = Quit;
        }
        if self.gui.text_button("Options", &self.glyph_atlas) == Released {
            println!("Options are not implemented");
        }
        if self.gui.text_button("New Game", &self.glyph_atlas) == Released
        {
            self.state = Play;
        }
    }

    fn update_game_gui(&mut self, input: &mut Input) {
        use DrawDirection::*;

        let player = &self.level.player;
        let level = player.stats.as_ref().unwrap().level;
        let health_ratio = player.get_health_ratio();
        let stamina_ratio = player.get_stamina_ratio();
        let exp_ratio = player.get_exp_ratio();

        self.gui.begin(input);

        self.gui.set_cursor_at_bot_left();

        self.gui.set_draw_direction(Right);
        self.gui.set_horizontal_padding(0.0);
        self.gui.rect_with_text(
            Vec2::new(77.0, 77.0),
            Color::expbar(exp_ratio),
            28,
            &level.to_string(),
            Color::black(1.0),
            &self.glyph_atlas,
        );

        self.gui.set_draw_direction(Up);
        self.gui.set_bar_size_scale(1.0, 0.3);
        self.gui.add_bar_size(10.0, 0.0);
        self.gui.bar(exp_ratio, Color::expbar(exp_ratio));

        self.gui.reset_horizontal_padding();
        self.gui.set_default_bar_size();
        self.gui.advance_cursor(10.0, 0.0);
        self.gui
            .bar(stamina_ratio, Color::staminabar(stamina_ratio));
        self.gui.bar(health_ratio, Color::healthbar(health_ratio));
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