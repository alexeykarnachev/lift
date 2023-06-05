use crate::components::*;
use crate::frame::*;
use crate::renderer::*;
use crate::vec::*;

#[derive(PartialEq, Debug, Clone, Copy)]
enum State {
    Idle,
}

pub struct Wolf {
    curr_state: State,
    next_state: State,

    move_speed: f32,

    krs: KinematicRigidSprite,
    animator: FrameAnimator,
}

impl Wolf {
    pub fn new(frame_atlas: FrameAtlas, position: Vec2<f32>) -> Self {
        Self {
            curr_state: State::Idle,
            next_state: State::Idle,
            move_speed: 100.0,
            krs: KinematicRigidSprite::new(position),
            animator: FrameAnimator::new(frame_atlas),
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        gravity: f32,
        rigid_colliders: &[Rect],
        renderer: &mut Renderer,
    ) {
        use State::*;

        match self.curr_state {
            Idle => {
                self.set_curr_state(Idle);
            }
        }

        if self.animator.is_finished() {
            self.set_curr_state(self.next_state);
        }

        let frame = self.animator.update(dt);
        self.krs
            .update(dt, gravity, frame, rigid_colliders, renderer);
    }

    fn set_curr_state(&mut self, state: State) {
        use State::*;

        self.curr_state = state;
        match self.curr_state {
            Idle => self.animator.play("wolf_idle", 0.07, true),
        }
    }

    fn set_next_state(&mut self, state: State) {
        self.next_state = state;
    }
}
