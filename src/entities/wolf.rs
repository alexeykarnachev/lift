use crate::components::*;
use crate::frame::*;
use crate::renderer::*;
use crate::vec::*;

#[derive(PartialEq, Debug, Clone, Copy)]
enum State {
    Idle,
    Run,
    AttackPrepare,
    AttackBite,
}

pub struct Wolf {
    curr_state: State,
    next_state: State,

    move_speed: f32,
    view_dist: f32,
    attack_dist: f32,

    krs: KinematicRigidSprite,
    animator: FrameAnimator,
}

impl Wolf {
    pub fn new(frame_atlas: FrameAtlas, position: Vec2<f32>) -> Self {
        Self {
            curr_state: State::Idle,
            next_state: State::Idle,
            move_speed: 50.0,
            view_dist: 200.0,
            attack_dist: 35.0,
            krs: KinematicRigidSprite::new(position),
            animator: FrameAnimator::new(frame_atlas),
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        gravity: f32,
        player_position: Vec2<f32>,
        rigid_colliders: &[Rect],
        renderer: &mut Renderer,
    ) {
        use State::*;

        let to_player = player_position - self.krs.position;
        let dist = to_player.len();
        let dir = to_player.x.signum();

        if dist <= self.view_dist {
            self.krs.look_dir = dir;
        }

        match self.curr_state {
            Idle => {
                self.set_curr_state(Idle);

                if dist < self.view_dist {
                    self.set_curr_state(Run);
                }
            }
            Run => {
                if dist <= self.attack_dist {
                    self.set_curr_state(AttackPrepare);
                } else if dist >= self.view_dist {
                    self.set_curr_state(Idle);
                } else {
                    self.krs.do_immediate_step(dt, self.move_speed, dir);
                }
            }
            AttackPrepare => {
                if dist > self.attack_dist {
                    self.set_next_state(Run);
                } else {
                    self.set_next_state(AttackBite);
                }
            }
            AttackBite => {
                if dist > self.attack_dist {
                    self.set_curr_state(Run);
                } else {
                    self.set_next_state(AttackPrepare);
                }
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
            Run => self.animator.play("wolf_run", 0.07, true),
            AttackPrepare => {
                self.animator.play("wolf_attack_prepare", 0.07, false)
            }
            AttackBite => {
                self.animator.play("wolf_attack_bite", 0.07, false)
            }
        }
    }

    fn set_next_state(&mut self, state: State) {
        self.next_state = state;
    }
}
