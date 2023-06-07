use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;

#[derive(Copy, Clone)]
enum Behaviour {
    KnightPlayer,
    WolfAI,
}

#[derive(Copy, Clone, Debug)]
enum State {
    Idle,
    Run,
}

const MAX_N_ENTITIES: usize = 1024;

pub struct Game {
    frame_atlas: &'static FrameAtlas,
    camera: Camera,
    gravity: f32,

    n_entities: usize,
    behaviours: [Option<Behaviour>; MAX_N_ENTITIES],
    states: [Option<State>; MAX_N_ENTITIES],
    positions: [Option<Vec2<f32>>; MAX_N_ENTITIES],
    move_speeds: [Option<f32>; MAX_N_ENTITIES],
    look_dirs: [Option<f32>; MAX_N_ENTITIES],

    frame_animators: [Option<FrameAnimator>; MAX_N_ENTITIES],
    rigid_colliders: [Option<Rect>; MAX_N_ENTITIES],
    attack_colliders: [Option<Rect>; MAX_N_ENTITIES],
    sprites: [Option<XYWH>; MAX_N_ENTITIES],
}

impl Game {
    pub fn new(frame_atlas_fp: &str) -> Self {
        let camera = Camera::new(Vec2::zeros());
        let frame_atlas = Box::new(FrameAtlas::new(frame_atlas_fp));

        Self {
            frame_atlas: Box::leak(frame_atlas),
            camera,
            gravity: 400.0,

            n_entities: 0,
            behaviours: [None; MAX_N_ENTITIES],
            states: [None; MAX_N_ENTITIES],
            positions: [None; MAX_N_ENTITIES],
            move_speeds: [None; MAX_N_ENTITIES],
            look_dirs: [None; MAX_N_ENTITIES],

            frame_animators: [None; MAX_N_ENTITIES],
            rigid_colliders: [None; MAX_N_ENTITIES],
            attack_colliders: [None; MAX_N_ENTITIES],
            sprites: [None; MAX_N_ENTITIES],
        }
    }

    pub fn new_knight_player(
        &mut self,
        position: Vec2<f32>,
    ) -> Option<usize> {
        if self.n_entities == MAX_N_ENTITIES {
            return None;
        }

        let idx = self.n_entities;
        self.n_entities += 1;

        self.behaviours[idx] = Some(Behaviour::KnightPlayer);
        self.states[idx] = Some(State::Idle);
        self.positions[idx] = Some(position);
        self.move_speeds[idx] = Some(100.0);
        self.frame_animators[idx] = Some(self.frame_atlas.new_animator());

        Some(idx)
    }

    pub fn new_wolf_ai(&mut self, position: Vec2<f32>) -> Option<usize> {
        if self.n_entities == MAX_N_ENTITIES {
            return None;
        }

        let idx = self.n_entities;
        self.n_entities += 1;

        self.behaviours[idx] = Some(Behaviour::WolfAI);
        self.states[idx] = Some(State::Idle);
        self.positions[idx] = Some(position);
        self.move_speeds[idx] = Some(100.0);
        self.frame_animators[idx] = Some(self.frame_atlas.new_animator());

        Some(idx)
    }

    pub fn update(
        &mut self,
        dt: f32,
        renderer: &mut Renderer,
        input: &mut Input,
    ) {
        renderer
            .set_camera(self.camera.position, self.camera.get_view_size());

        self.update_behaviours(dt, input);
        self.update_frame_animators(dt);
        self.update_renderer(renderer);
    }

    fn update_behaviours(&mut self, dt: f32, input: &Input) {
        use Behaviour::*;

        for idx in 0..self.n_entities {
            match self.behaviours[idx] {
                Some(KnightPlayer) => {
                    self.update_knight_player_behaviour(dt, idx, input);
                }
                Some(WolfAI) => {
                    self.update_wolf_ai_behaviour(idx);
                }
                None => {}
            }
        }
    }

    fn update_knight_player_behaviour(
        &mut self,
        dt: f32,
        idx: usize,
        input: &Input,
    ) {
        use sdl2::keyboard::Keycode::*;
        use State::*;

        let is_left_action = input.key_is_down(A);
        let is_right_action = input.key_is_down(D);
        let is_step_action = is_right_action || is_left_action;
        let dir = if is_right_action { 1.0 } else { -1.0 };

        let state = &mut self.states[idx];

        match state {
            Some(Idle) => {
                if is_step_action {
                    *state = Some(Run);
                }
            }
            Some(Run) => {
                if is_step_action {
                    self.do_immediate_step(dt, idx, dir);
                }
            }
            None => {
                self.states[idx] = Some(Idle);
            }
        }
    }

    fn update_wolf_ai_behaviour(&mut self, idx: usize) {
        use State::*;

        match self.states[idx] {
            Some(Idle) => {}
            None => {
                self.states[idx] = Some(Idle);
            }
            _ => {}
        }
    }

    fn update_frame_animators(&mut self, dt: f32) {
        use Behaviour::*;
        use State::*;

        for idx in 0..self.n_entities {
            if let (Some(animator), Some(position)) =
                (self.frame_animators[idx].as_mut(), self.positions[idx])
            {
                if let Some(frame) = animator.update(dt) {
                    self.sprites[idx] = Some(frame.sprite);
                    self.rigid_colliders[idx] = frame.get_mask(
                        "rigid",
                        Pivot::BotCenter(position),
                        false,
                    );
                    self.attack_colliders[idx] = frame.get_mask(
                        "attack",
                        Pivot::BotCenter(position),
                        false,
                    );
                }
            }

            if let (Some(animator), Some(behaviour), Some(state)) = (
                self.frame_animators[idx].as_mut(),
                self.behaviours[idx],
                self.states[idx],
            ) {
                match (behaviour, state) {
                    (WolfAI, Idle) => {
                        animator.play("wolf_idle", 0.07, true);
                    }
                    (KnightPlayer, Idle) => {
                        animator.play("knight_idle", 0.07, true);
                    }
                    _ => {}
                }
            }
        }
    }

    fn update_renderer(&mut self, renderer: &mut Renderer) {
        for idx in 0..self.n_entities {
            if let (Some(sprite), Some(position)) =
                (self.sprites[idx], self.positions[idx])
            {
                let pivot = Pivot::BotCenter(position);
                let apply_light = false;
                let flip = false;
                let primitive = DrawPrimitive::world_sprite(
                    sprite,
                    pivot,
                    apply_light,
                    flip,
                );

                renderer.push_primitive(primitive);
            }
        }
    }

    fn do_immediate_step(&mut self, dt: f32, idx: usize, dir: f32) {
        self.look_dirs[idx] = Some(dir);
        if let (Some(position), Some(speed)) =
            (self.positions[idx].as_mut(), self.move_speeds[idx])
        {
            position.x += dt * speed * dir;
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
