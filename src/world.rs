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
    MainMenu,
    Play,
    GameOver,
    SkillTree,
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
    pub attacks: Vec<Attack>,

    pub main_menu_ui: UI,
    pub play_ui: UI,
    pub game_over_ui: UI,
    pub skill_tree_ui: UI,

    pub sprite_atlas: SpriteAtlas,
    pub glyph_atlas: GlyphAtlas,
}

impl World {
    pub fn new() -> Self {
        let sprite_atlas = create_default_sprite_atlas();
        let glyph_atlas = create_default_glyph_atlas();
        let level = Level::new("./assets/levels/0.json", &sprite_atlas);

        let attacks: Vec<Attack> = Vec::with_capacity(256);
        let camera = Camera::new(level.player.get_center().add_y(2.0));

        let main_menu_ui = create_default_main_menu_ui();
        let play_ui = create_play_ui();
        let game_over_ui = create_default_game_over_ui();
        let skill_tree_ui = create_skill_tree_ui(
            &sprite_atlas,
            level.player.stats.as_ref().unwrap(),
        );

        Self {
            // state: WorldState::MainMenu,
            state: WorldState::Play,
            time: 0.0,
            gravity: 400.0,
            friction: 0.02,
            level,
            camera,
            attacks,
            main_menu_ui,
            play_ui,
            game_over_ui,
            skill_tree_ui,
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
                self.update_main_menu_ui(input);
            }
            Play => {
                if input.take_action(Keyaction::SkillTree) {
                    self.state = SkillTree;
                } else {
                    self.update_play_ui(input);
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
                self.update_game_over_ui(input);
            }
            SkillTree => {
                if input.take_action(Keyaction::SkillTree) {
                    self.state = Play;
                } else {
                    self.update_skill_tree_ui(input);
                }
            }
            Restart => {
                *self = Self::new();
                self.state = Play;
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
                                enemy.state = Running;
                            }
                        }
                    }
                    Running => {
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
                                    enemy.state = Running;
                                }
                            }
                        }
                        Running => {
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
                                enemy.state = Running;
                            }
                        }
                    }
                    Running => {
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
                    player.state = Running;
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
            Running => {
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
                        player.play_animation("run");
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
                player.play_animation("slide");
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

                attack.damage /= attacked_enemies.len() as f32;
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

    fn update_main_menu_ui(&mut self, input: &Input) {
        use UIEvent::*;

        let events = self.main_menu_ui.update(
            input,
            &self.glyph_atlas,
            &self.sprite_atlas,
        );
        for event in events.iter() {
            match event {
                LMBPress(id) => match id.as_str() {
                    "new_game" => self.state = WorldState::Play,
                    "quit" => {
                        self.state = WorldState::Quit;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn update_play_ui(&mut self, input: &Input) {
        let player = &self.level.player;
        let stats = player.stats.as_ref().unwrap();
        let health_ratio = player.get_health_ratio();
        let stamina_ratio = player.get_stamina_ratio();
        let exp_ratio = player.get_exp_ratio();

        self.play_ui.set_element_color(
            "healthbar",
            Color::healthbar(health_ratio),
        );
        self.play_ui.set_element_filling("healthbar", health_ratio);

        self.play_ui.set_element_color(
            "staminabar",
            Color::staminabar(stamina_ratio),
        );
        self.play_ui
            .set_element_filling("staminabar", stamina_ratio);

        self.play_ui
            .set_element_color("expbar", Color::expbar(exp_ratio));
        self.play_ui.set_element_filling("expbar", exp_ratio);

        self.play_ui.set_element_color(
            "level_number_rect",
            Color::expbar(exp_ratio.max(0.05)),
        );

        let level = stats.level.to_string();
        self.play_ui.set_element_string("level_number", &level);

        _ = self.play_ui.update(
            input,
            &self.glyph_atlas,
            &self.sprite_atlas,
        );
    }

    fn update_game_over_ui(&mut self, input: &Input) {
        use UIEvent::*;

        let events = self.game_over_ui.update(
            input,
            &self.glyph_atlas,
            &self.sprite_atlas,
        );
        for event in events.iter() {
            match event {
                LMBPress(id) => match id.as_str() {
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

    fn update_skill_tree_ui(&mut self, input: &Input) {
        use EffectType::*;
        use UIEvent::*;
        let alpha01 = AlphaEffect01 as u32;
        let alpha03 = AlphaEffect03 as u32;
        let alpha07 = AlphaEffect07 as u32;

        let events = self.skill_tree_ui.update(
            input,
            &self.glyph_atlas,
            &self.sprite_atlas,
        );

        let stats = self.level.player.stats.as_mut().unwrap();
        let n_points = stats.n_skill_points;

        self.skill_tree_ui.set_element_string(
            "skill_points_text",
            &format!("POINTS: {:?}", n_points),
        );

        let mut learned_skills_name = None;
        for event in events.iter() {
            let learned_sprite_effect = 0;
            let learned_arrow_effect = 0;
            let mut can_learn_sprite_effect = 0;
            let mut can_learn_arrow_effect = 0;
            let cant_learn_sprite_effect = alpha01 | alpha03;
            let cant_learn_arrow_effect = alpha01 | alpha03;
            let mut lmb_is_pressed = false;

            let mut sprite_id: Option<String> = None;
            match event {
                Hover(id) => {
                    sprite_id = Some(id.clone());
                    can_learn_sprite_effect = alpha07;
                    can_learn_arrow_effect = alpha07;
                }
                NotInteracted(id) => {
                    sprite_id = Some(id.clone());
                    can_learn_sprite_effect = alpha03;
                    can_learn_arrow_effect = alpha03;
                }
                LMBPress(id) => {
                    sprite_id = Some(id.clone());
                    lmb_is_pressed = true;
                    can_learn_sprite_effect = alpha07;
                    can_learn_arrow_effect = alpha07;
                }
                _ => {}
            }

            if let Some(sprite_id) = sprite_id {
                let (name, idx) = sprite_id.rsplit_once("_").unwrap();
                let idx = usize::from_str_radix(idx, 10).unwrap();
                let mut skills = stats.get_skills_by_name(name);
                let n_learned = skills.n_learned;
                let mut arrow_id = "arrow_".to_string();
                arrow_id.push_str(&sprite_id);

                let is_learned = idx < n_learned;
                let can_learn = idx == n_learned && n_points > 0;
                let cant_learn = idx > n_learned || n_points == 0;
                let has_arrow = idx > 0;

                if lmb_is_pressed && can_learn {
                    learned_skills_name = Some(name.to_string());
                }

                if is_learned {
                    self.skill_tree_ui.set_element_effect(
                        &sprite_id,
                        learned_sprite_effect,
                    );
                } else if can_learn {
                    self.skill_tree_ui.set_element_effect(
                        &sprite_id,
                        can_learn_sprite_effect,
                    );
                } else if cant_learn {
                    self.skill_tree_ui.set_element_effect(
                        &sprite_id,
                        cant_learn_sprite_effect,
                    );
                }

                if has_arrow && is_learned {
                    self.skill_tree_ui.set_element_effect(
                        &arrow_id,
                        learned_arrow_effect,
                    );
                } else if has_arrow && can_learn {
                    self.skill_tree_ui.set_element_effect(
                        &arrow_id,
                        can_learn_arrow_effect,
                    );
                } else if has_arrow && cant_learn {
                    self.skill_tree_ui.set_element_effect(
                        &arrow_id,
                        cant_learn_arrow_effect,
                    );
                }
            }
        }

        if let Some(name) = learned_skills_name {
            let skills = stats.get_skills_by_name(&name);
            skills.n_learned += 1;
            stats.n_skill_points -= 1;
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
