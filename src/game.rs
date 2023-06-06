use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;

#[derive(Copy, Clone)]
enum Behaviour {
    KnightPlayer,
    WolfAI,
}

#[derive(Copy, Clone)]
enum State {
    Idle,
    Run,
}

const MAX_N_ENTITIES: usize = 1024;

pub struct Game {
    camera: Camera,
    gravity: f32,

    n_entities: usize,
    behaviours: [Option<Behaviour>; MAX_N_ENTITIES],
    states: [Option<State>; MAX_N_ENTITIES],
    positions: [Option<Vec2<f32>>; MAX_N_ENTITIES],
    move_speeds: [Option<f32>; MAX_N_ENTITIES],

    frame_animators: [Option<FrameAnimator>; MAX_N_ENTITIES],
    rigid_colliders: [Option<Rect>; MAX_N_ENTITIES],
    attack_colliders: [Option<Rect>; MAX_N_ENTITIES],
    sprites: [Option<XYWH>; MAX_N_ENTITIES],
}

impl Game {
    pub fn new() -> Self {
        let camera = Camera::new(Vec2::zeros());

        Self {
            camera,
            gravity: 400.0,

            n_entities: 0,
            behaviours: [None; MAX_N_ENTITIES],
            states: [None; MAX_N_ENTITIES],
            positions: [None; MAX_N_ENTITIES],
            move_speeds: [None; MAX_N_ENTITIES],

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

        Some(idx)
    }

    pub fn new_wolf_ai(&mut self, position: Vec2<f32>, frame_atlas: &'static FrameAtlas) -> Option<usize> {
        if self.n_entities == MAX_N_ENTITIES {
            return None;
        }

        let idx = self.n_entities;
        self.n_entities += 1;

        self.behaviours[idx] = Some(Behaviour::WolfAI);
        self.states[idx] = Some(State::Idle);
        self.positions[idx] = Some(position);
        self.move_speeds[idx] = Some(100.0);

        self.frame_animators[idx] = Some(FrameAnimator::new(frame_atlas));

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

        self.update_frame_animators(dt);
        self.update_renderer(renderer);
    }

    fn update_frame_animators(&mut self, dt: f32) {
        for idx in 0..self.n_entities {
            if let (Some(animator), Some(position)) = (self.frame_animators[idx].as_mut(), self.positions[idx]) {
                let frame = animator.update(dt);
                self.sprites[idx] = Some(frame.sprite);
                self.rigid_colliders[idx] = frame.get_mask("rigid", Pivot::BotCenter(position), false);
                self.attack_colliders[idx] = frame.get_mask("attack", Pivot::BotCenter(position), false);
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
