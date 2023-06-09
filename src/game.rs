use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;
use sdl2::EventPump;
use std::time::Instant;

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

#[derive(Default)]
struct Debug {
    show_rigid_colliders: bool,
}

enum Behaviour {
    Static,
    KnightPlayerBehaviour(KnightPlayer),
    WolfAIBehaviour(WolfAI),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum KnightPlayerState {
    Idle,
    Run,
    Roll,
    Attack0,
    Attack1,
    Attack2,
    JumpUp,
    JumpDown,
    JumpLanding,
}

struct KnightPlayer {
    pub curr_state: KnightPlayerState,
    pub next_state: KnightPlayerState,
    pub can_perform_combo: bool,
    pub is_attack2_step_done: bool,
    pub run_speed: f32,
    pub roll_speed: f32,
    pub jump_speed: f32,
    pub landing_speed: f32,
    pub attack2_step: f32,
}

impl KnightPlayer {
    pub fn new(
        run_speed: f32,
        roll_speed: f32,
        jump_speed: f32,
        landing_speed: f32,
        attack2_step: f32,
    ) -> Self {
        Self {
            curr_state: KnightPlayerState::Idle,
            next_state: KnightPlayerState::Idle,
            can_perform_combo: false,
            is_attack2_step_done: false,
            run_speed,
            roll_speed,
            jump_speed,
            landing_speed,
            attack2_step,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum WolfAIState {
    Idle,
    Run,
    Attack,
}

struct WolfAI {
    pub curr_state: WolfAIState,
    pub next_state: WolfAIState,
}

impl WolfAI {
    pub fn new() -> Self {
        Self {
            curr_state: WolfAIState::Idle,
            next_state: WolfAIState::Idle,
        }
    }
}

#[derive(Clone, Copy)]
struct Kinematic {
    velocity: Vec2<f32>,
    is_grounded: bool,
}

impl Kinematic {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::zeros(),
            is_grounded: false,
        }
    }
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
    behaviours: [Behaviour; MAX_N_ENTITIES],
    look_dirs: [f32; MAX_N_ENTITIES],

    frame_animators: [Option<FrameAnimator>; MAX_N_ENTITIES],
    kinematics: [Option<Kinematic>; MAX_N_ENTITIES],
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
            behaviours: [(); MAX_N_ENTITIES].map(|_| Behaviour::Static),
            look_dirs: [1.0; MAX_N_ENTITIES],

            frame_animators: [(); MAX_N_ENTITIES].map(|_| None),
            kinematics: [(); MAX_N_ENTITIES].map(|_| None),
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

            if let (Some(mut rect), true) = (
                self.rigid_colliders[idx],
                self.debug.show_rigid_colliders,
            ) {
                rect = rect.translate(self.positions[idx]);
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
                KnightPlayerBehaviour(ref mut knight) => {
                    update_knight_player(
                        knight,
                        self.input,
                        self.dt,
                        &mut self.positions[idx],
                        &mut self.kinematics[idx].as_mut().unwrap(),
                        &mut self.frame_animators[idx].as_mut().unwrap(),
                        &mut self.look_dirs[idx],
                    );
                }
                WolfAIBehaviour(ref mut wolf) => {
                    update_wolf_ai(
                        wolf,
                        &mut self.kinematics[idx].as_mut().unwrap(),
                        &mut self.frame_animators[idx].as_mut().unwrap(),
                        &mut self.look_dirs[idx],
                    );
                }
                Static => {}
            }
        }
    }

    fn update_kinematics(&mut self) {
        for idx in 0..self.n_entities {
            let mut kinematic =
                if let Some(kinematic) = self.kinematics[idx] {
                    kinematic
                } else {
                    continue;
                };

            kinematic.velocity.y -= self.gravity * self.dt;
            self.positions[idx] += kinematic.velocity.scale(self.dt);

            if let Some(mut collider) = self.rigid_colliders[idx] {
                collider = collider.translate(self.positions[idx]);
                let mut is_grounded = false;

                for other_idx in 0..self.n_entities {
                    if idx == other_idx {
                        continue;
                    }

                    if let (Some(mut other_collider), None) = (
                        self.rigid_colliders[other_idx],
                        self.kinematics[other_idx],
                    ) {
                        other_collider = other_collider
                            .translate(self.positions[other_idx]);
                        let mtv = collider.collide_aabb(other_collider);
                        self.positions[idx] += mtv;

                        if mtv.y.abs() > 0.0 {
                            kinematic.velocity.y = 0.0;
                        }

                        if mtv.x.abs() > 0.0 {
                            kinematic.velocity.x = 0.0;
                        }

                        is_grounded |= mtv.y > 0.0;
                    }
                }

                kinematic.is_grounded = is_grounded;
            }

            self.kinematics[idx] = Some(kinematic);
        }
    }

    fn update_frame_animators(&mut self) {
        for idx in 0..self.n_entities {
            let mut animator =
                if let Some(animator) = self.frame_animators[idx] {
                    animator
                } else {
                    continue;
                };

            if let Some(frame) = animator.update(self.dt) {
                let position = self.positions[idx];
                let flip = self.look_dirs[idx] < 0.0;
                self.sprites[idx] = Some(frame.sprite);
                self.rigid_colliders[idx] = frame.get_mask(
                    "rigid",
                    Pivot::BotCenter(Vec2::zeros()),
                    flip,
                );
                self.attack_colliders[idx] = frame.get_mask(
                    "attack",
                    Pivot::BotCenter(Vec2::zeros()),
                    flip,
                );
            }

            self.frame_animators[idx] = Some(animator);
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
            let knight_player =
                KnightPlayer::new(100.0, 150.0, 150.0, 70.0, 8.0);
            let behaviour =
                Behaviour::KnightPlayerBehaviour(knight_player);

            self.positions[idx] = position;
            self.behaviours[idx] = behaviour;
            self.frame_animators[idx] =
                Some(self.frame_atlas.new_animator());
            self.kinematics[idx] = Some(Kinematic::new());
        }
    }

    pub fn new_wolf_ai(&mut self, position: Vec2<f32>) {
        if let Some(idx) = self.new_entity() {
            let wolf_ai = WolfAI::new();
            let behaviour = Behaviour::WolfAIBehaviour(wolf_ai);

            self.positions[idx] = position;
            self.behaviours[idx] = behaviour;
            self.frame_animators[idx] =
                Some(self.frame_atlas.new_animator());
            self.kinematics[idx] = Some(Kinematic::new());
        }
    }

    pub fn new_rigid_collider(&mut self, pivot: Pivot, size: Vec2<f32>) {
        if let Some(idx) = self.new_entity() {
            let rect = Rect::from_pivot(pivot, size);
            self.rigid_colliders[idx] = Some(rect);
        }
    }
}

fn update_knight_player(
    knight: &mut KnightPlayer,
    input: &mut Input,
    dt: f32,
    position: &mut Vec2<f32>,
    kinematic: &mut Kinematic,
    animator: &mut FrameAnimator,
    look_dir: &mut f32,
) {
    use sdl2::keyboard::Keycode::*;
    use KnightPlayerState::*;

    let is_attack_action = input.key_is_pressed(Space);
    let is_left_action = input.key_is_down(A);
    let is_right_action = input.key_is_down(D);
    let is_jump_action = input.key_is_pressed(W);
    let is_roll_action = input.key_is_pressed(LCtrl);
    let is_step_action = is_right_action || is_left_action;
    let dir = if is_right_action { 1.0 } else { -1.0 };

    if animator.is_finished() {
        knight.curr_state = knight.next_state;
        knight.next_state = Idle;
    }

    match knight.curr_state {
        Idle => {
            if is_attack_action {
                knight.curr_state = Attack0;
                knight.next_state = Idle;
                knight.can_perform_combo = true;
            } else if is_jump_action {
                knight.curr_state = JumpUp;
                kinematic.velocity.y += knight.jump_speed;
            } else if is_step_action {
                knight.curr_state = Run;
            }
        }
        Attack0 => {
            knight.is_attack2_step_done = false;
            if is_attack_action && knight.can_perform_combo {
                if animator.progress > 0.7 {
                    knight.next_state = Attack1;
                } else {
                    knight.can_perform_combo = false;
                }
            }
        }
        Attack1 => {
            if is_attack_action && knight.can_perform_combo {
                if animator.progress > 0.7 {
                    knight.next_state = Attack2;
                } else {
                    knight.can_perform_combo = false;
                }
            }
        }
        Attack2 => {
            knight.next_state = Idle;
            if !knight.is_attack2_step_done {
                position.x += *look_dir * knight.attack2_step;
                knight.is_attack2_step_done = true;
            }
        }
        Run => {
            if is_roll_action {
                knight.curr_state = Roll;
            } else if is_jump_action {
                knight.curr_state = JumpUp;
                kinematic.velocity.y += knight.jump_speed;
            } else if is_attack_action {
                knight.curr_state = Attack0;
                knight.can_perform_combo = true;
            } else if is_step_action {
                position.x += dir * dt * knight.run_speed;
                *look_dir = dir;
            } else {
                knight.curr_state = Idle;
            }
        }
        Roll => {
            knight.next_state = Idle;
            let speed =
                knight.roll_speed * (1.0 - animator.progress.powf(2.0));
            position.x += *look_dir * dt * speed;
        }
        JumpUp => {
            if kinematic.velocity.y > 0.0 {
                knight.next_state = JumpUp;
            } else {
                knight.curr_state = JumpDown;
            }

            if is_step_action {
                position.x += dir * dt * knight.run_speed;
                *look_dir = dir;
            }
        }
        JumpDown => {
            if !kinematic.is_grounded {
                knight.next_state = JumpDown;
            } else {
                knight.curr_state = JumpLanding;
            }

            if is_step_action {
                position.x += dir * dt * knight.run_speed;
                *look_dir = dir;
            }
        }
        JumpLanding => {
            knight.next_state = Idle;
            if is_step_action {
                position.x += dir * dt * knight.landing_speed;
                *look_dir = dir;
            }
        }
    }

    match knight.curr_state {
        Idle => animator.play("knight_idle", 0.07, true),
        Run => animator.play("knight_run", 0.07, true),
        Roll => animator.play("knight_roll", 0.07, false),
        JumpUp => animator.play("knight_jump_up", 0.07, false),
        JumpDown => animator.play("knight_jump_down", 0.07, false),
        JumpLanding => animator.play("knight_jump_landing", 0.07, false),
        Attack0 => animator.play("knight_attack_0", 0.07, false),
        Attack1 => animator.play("knight_attack_1", 0.07, false),
        Attack2 => animator.play("knight_attack_2", 0.07, false),
    }
}

fn update_wolf_ai(
    wolf: &mut WolfAI,
    kinematic: &mut Kinematic,
    animator: &mut FrameAnimator,
    look_dir: &mut f32,
) {
    use WolfAIState::*;

    if animator.is_finished() {
        wolf.curr_state = wolf.next_state;
    }

    match wolf.curr_state {
        Idle => {}
        _ => {}
    }

    match wolf.curr_state {
        Idle => animator.play("wolf_idle", 0.07, true),
        Run => animator.play("wolf_run", 0.07, true),
        Attack => animator.play("wolf_attack", 0.07, false),
    }
}
