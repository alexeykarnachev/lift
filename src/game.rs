use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;
use sdl2::EventPump;
use std::time::Instant;

#[derive(Copy, Clone)]
enum Behaviour {
    Static,
    KnightPlayer,
    WolfAI,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    Idle,
    Run,
    Roll,
    JumpUp,
    JumpDown,
    JumpLanding,
}

#[derive(Default)]
struct Debug {
    show_rigid_colliders: bool,
}

const MAX_N_ENTITIES: usize = 1024;

pub struct Game {
    dt: f32,
    prev_upd_time: Instant,

    event_pump: &'static mut EventPump,
    input: &'static mut Input,
    frame_atlas: &'static FrameAtlas,
    renderer: &'static mut Renderer,
    camera: Camera,

    gravity: f32,

    n_entities: usize,
    positions: [Vec2<f32>; MAX_N_ENTITIES],
    velocities: [Vec2<f32>; MAX_N_ENTITIES],
    curr_states: [State; MAX_N_ENTITIES],
    next_states: [State; MAX_N_ENTITIES],
    behaviours: [Behaviour; MAX_N_ENTITIES],
    look_dirs: [f32; MAX_N_ENTITIES],
    are_grounded: [bool; MAX_N_ENTITIES],
    are_kinematic: [bool; MAX_N_ENTITIES],

    frame_animators: [Option<FrameAnimator>; MAX_N_ENTITIES],
    rigid_colliders: [Option<Rect>; MAX_N_ENTITIES],
    attack_colliders: [Option<Rect>; MAX_N_ENTITIES],
    sprites: [Option<XYWH>; MAX_N_ENTITIES],

    debug: Debug,
}

impl Game {
    pub fn new(
        window_size: Vec2<u32>,
        frame_atlas_meta_fp: &str,
        frame_atlas_image_fp: &str,
    ) -> Self {
        let camera = Camera::new(Vec2::zeros());
        let frame_atlas = Box::new(FrameAtlas::new(frame_atlas_meta_fp));
        let input = Box::new(Input::new(window_size));

        let sdl = sdl2::init().unwrap();
        let event_pump = Box::new(sdl.event_pump().unwrap());
        let renderer = Box::new(Renderer::new(
            &sdl,
            "Lift",
            window_size,
            frame_atlas_image_fp,
        ));

        let debug = Debug {
            show_rigid_colliders: true,
        };

        Self {
            dt: 0.0,
            prev_upd_time: Instant::now(),
            event_pump: Box::leak(event_pump),
            input: Box::leak(input),
            frame_atlas: Box::leak(frame_atlas),
            renderer: Box::leak(renderer),
            camera,
            gravity: 400.0,

            n_entities: 0,
            positions: [Vec2::zeros(); MAX_N_ENTITIES],
            velocities: [Vec2::zeros(); MAX_N_ENTITIES],
            behaviours: [Behaviour::Static; MAX_N_ENTITIES],
            curr_states: [State::Idle; MAX_N_ENTITIES],
            next_states: [State::Idle; MAX_N_ENTITIES],
            look_dirs: [1.0; MAX_N_ENTITIES],
            are_grounded: [false; MAX_N_ENTITIES],
            are_kinematic: [false; MAX_N_ENTITIES],

            frame_animators: [None; MAX_N_ENTITIES],
            rigid_colliders: [None; MAX_N_ENTITIES],
            attack_colliders: [None; MAX_N_ENTITIES],
            sprites: [None; MAX_N_ENTITIES],

            debug,
        }
    }

    pub fn start(&mut self) {
        self.new_knight_player(Vec2::new(0.0, 0.0));
        self.new_wolf_ai(Vec2::new(40.0, 0.0));
        self.new_rigid_collider(
            Pivot::TopCenter(Vec2::new(0.0, 0.0)),
            Vec2::new(1000.0, 50.0),
        );

        while !self.input.should_quit {
            self.update_input();
            self.update_world();
            self.update_renderer();
        }
    }

    pub fn get_rigid_collider(&mut self, idx: usize) -> Option<Rect> {
        if let Some(rect) = self.rigid_colliders[idx] {
            return Some(rect.translate(self.positions[idx]));
        }

        None
    }

    pub fn get_attack_collider(&mut self, idx: usize) -> Option<Rect> {
        if let Some(rect) = self.attack_colliders[idx] {
            return Some(rect.translate(self.positions[idx]));
        }

        None
    }

    fn update_input(&mut self) {
        for event in self.event_pump.poll_iter() {
            self.input.handle_event(&event);
        }
        self.input.update();
    }

    fn update_world(&mut self) {
        self.dt = self.prev_upd_time.elapsed().as_nanos() as f32 / 1.0e9;
        self.update_behaviours();
        self.update_frame_animators();
        self.update_kinematics();
        self.prev_upd_time = Instant::now();
    }

    fn update_renderer(&mut self) {
        self.renderer.clear_queue();
        self.renderer
            .set_camera(self.camera.position, self.camera.get_view_size());

        for idx in 0..self.n_entities {
            if let Some(sprite) = self.sprites[idx] {
                let position = self.positions[idx];
                let pivot = Pivot::BotCenter(position);
                let apply_light = false;
                let flip = self.look_dirs[idx] < 0.0;
                let primitive = DrawPrimitive::world_sprite(
                    sprite,
                    pivot,
                    apply_light,
                    flip,
                );

                self.renderer.push_primitive(primitive);
            }

            if let (Some(rect), true) = (
                self.get_rigid_collider(idx),
                self.debug.show_rigid_colliders,
            ) {
                let primitive =
                    DrawPrimitive::world_rect(rect, Color::red(0.2));
                self.renderer.push_primitive(primitive);
            }
        }

        self.renderer.render();
    }

    fn update_behaviours(&mut self) {
        use Behaviour::*;

        for idx in 0..self.n_entities {
            match self.behaviours[idx] {
                KnightPlayer => {
                    self.update_knight_player_behaviour(idx);
                }
                WolfAI => {
                    self.update_wolf_ai_behaviour(idx);
                }
                Static => {}
            }
        }
    }

    fn update_kinematics(&mut self) {
        for idx in 0..self.n_entities {
            if !self.are_kinematic[idx] {
                continue;
            }

            self.velocities[idx].y -= self.gravity * self.dt;
            self.positions[idx] += self.velocities[idx].scale(self.dt);

            if let Some(collider) = self.get_rigid_collider(idx) {
                let mut is_grounded = false;

                for other_idx in 0..self.n_entities {
                    if idx == other_idx {
                        continue;
                    }

                    if let (Some(other_collider), false) = (
                        self.get_rigid_collider(other_idx),
                        self.are_kinematic[other_idx],
                    ) {
                        let mtv = collider.collide_aabb(other_collider);
                        self.positions[idx] += mtv;

                        if mtv.y.abs() > 0.0 {
                            self.velocities[idx].y = 0.0;
                        }

                        if mtv.x.abs() > 0.0 {
                            self.velocities[idx].x = 0.0;
                        }

                        is_grounded |= mtv.y > 0.0;
                    }
                }

                self.are_grounded[idx] = is_grounded;
            }
        }
    }

    fn update_knight_player_behaviour(&mut self, idx: usize) {
        use sdl2::keyboard::Keycode::*;
        use State::*;

        let jump_speed = 150.0;

        let is_jump_action = self.input.key_is_pressed(W);
        let is_roll_action = self.input.key_is_pressed(LCtrl);
        let is_left_action = self.input.key_is_down(A);
        let is_right_action = self.input.key_is_down(D);
        let is_step_action = is_right_action || is_left_action;
        let dir = if is_right_action { 1.0 } else { -1.0 };

        let animator = self.frame_animators[idx].as_ref().expect(
            "Kinight Player should have the FrameAnimator component",
        );
        match self.curr_states[idx] {
            Idle => {
                if is_jump_action {
                    self.set_curr_state(idx, JumpUp);
                    self.velocities[idx].y += jump_speed;
                } else if is_step_action {
                    self.set_curr_state(idx, Run);
                }
            }
            Run => {
                if is_roll_action {
                    self.set_curr_state(idx, Roll);
                } else if is_jump_action {
                    self.set_curr_state(idx, JumpUp);
                    self.velocities[idx].y += jump_speed;
                } else if is_step_action {
                    self.do_immediate_step(idx, 100.0, dir);
                } else {
                    self.set_curr_state(idx, Idle);
                }
            }
            Roll => {
                let speed = 150.0 * (1.0 - animator.progress.powf(2.0));
                self.do_immediate_step(idx, speed, self.look_dirs[idx]);
            }
            JumpUp => {
                if self.velocities[idx].y > 0.0 {
                    self.set_next_state(idx, JumpUp);
                } else {
                    self.set_next_state(idx, JumpDown);
                }

                if is_step_action {
                    self.do_immediate_step(idx, 100.0, dir)
                }
            }
            JumpDown => {
                if self.are_grounded[idx] {
                    self.set_next_state(idx, JumpLanding);
                } else {
                    self.set_next_state(idx, JumpDown);
                }

                if is_step_action {
                    self.do_immediate_step(idx, 100.0, dir);
                }
            }
            JumpLanding => {
                if is_step_action {
                    self.do_immediate_step(idx, 70.0, dir);
                }
            }
        }
    }

    fn update_wolf_ai_behaviour(&mut self, idx: usize) {
        use State::*;

        match self.curr_states[idx] {
            Idle => {}
            _ => {}
        }
    }

    fn update_frame_animators(&mut self) {
        use Behaviour::*;
        use State::*;

        for idx in 0..self.n_entities {
            let animator = if let Some(animator) =
                self.frame_animators[idx].as_mut()
            {
                animator
            } else {
                continue;
            };

            if let Some(frame) = animator.update(self.dt) {
                let position = self.positions[idx];
                self.sprites[idx] = Some(frame.sprite);
                self.rigid_colliders[idx] = frame.get_mask(
                    "rigid",
                    Pivot::BotCenter(Vec2::zeros()),
                    false,
                );
                self.attack_colliders[idx] = frame.get_mask(
                    "attack",
                    Pivot::BotCenter(Vec2::zeros()),
                    false,
                );
            }

            match (self.behaviours[idx], self.curr_states[idx]) {
                (WolfAI, Idle) => {
                    animator.play("wolf_idle", 0.07, true);
                }
                (KnightPlayer, Idle) => {
                    animator.play("knight_idle", 0.07, true);
                }
                (KnightPlayer, Run) => {
                    animator.play("knight_run", 0.07, true);
                }
                (KnightPlayer, Roll) => {
                    animator.play("knight_roll", 0.07, false);
                }
                (KnightPlayer, JumpUp) => {
                    animator.play("knight_jump_up", 0.07, false);
                }
                (KnightPlayer, JumpDown) => {
                    animator.play("knight_jump_down", 0.07, false);
                }
                (KnightPlayer, JumpLanding) => {
                    animator.play("knight_jump_landing", 0.07, false);
                }
                _ => {}
            }

            if animator.is_finished() {
                self.set_curr_state(idx, self.next_states[idx]);
            }
        }
    }

    fn new_entity(&mut self) -> Option<usize> {
        if self.n_entities == MAX_N_ENTITIES {
            return None;
        }

        let idx = self.n_entities;
        self.n_entities += 1;

        return Some(idx);
    }

    fn new_knight_player(&mut self, position: Vec2<f32>) {
        if let Some(idx) = self.new_entity() {
            self.positions[idx] = position;
            self.behaviours[idx] = Behaviour::KnightPlayer;
            self.curr_states[idx] = State::Idle;
            self.frame_animators[idx] =
                Some(self.frame_atlas.new_animator());
            self.are_kinematic[idx] = true;
        }
    }

    pub fn new_wolf_ai(&mut self, position: Vec2<f32>) {
        if let Some(idx) = self.new_entity() {
            self.positions[idx] = position;
            self.behaviours[idx] = Behaviour::WolfAI;
            self.curr_states[idx] = State::Idle;
            self.frame_animators[idx] =
                Some(self.frame_atlas.new_animator());
            self.are_kinematic[idx] = true;
        }
    }

    pub fn new_rigid_collider(&mut self, pivot: Pivot, size: Vec2<f32>) {
        if let Some(idx) = self.new_entity() {
            let rect = Rect::from_pivot(pivot, size);
            self.rigid_colliders[idx] = Some(rect);
        }
    }

    fn do_immediate_step(&mut self, idx: usize, speed: f32, dir: f32) {
        self.look_dirs[idx] = dir;
        self.positions[idx].x += self.dt * speed * dir;
    }

    fn set_curr_state(&mut self, idx: usize, state: State) {
        if self.curr_states[idx] != state {
            self.next_states[idx] = State::Idle;
        }

        self.curr_states[idx] = state;
    }

    fn set_next_state(&mut self, idx: usize, state: State) {
        self.next_states[idx] = state;
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
