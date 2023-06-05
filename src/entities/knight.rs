use crate::components::KinematicRigidSprite;
use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;

#[derive(PartialEq, Debug, Clone, Copy)]
enum State {
    Idle,
    Run,
    Roll,
    JumpUp,
    JumpDown,
    JumpLanding,
    Attack0,
    Attack1,
    Attack2,
}

pub struct Knight {
    curr_state: State,
    next_state: State,
    can_perform_combo: bool,
    is_attack2_step_done: bool,

    move_speed: f32,
    jump_speed: f32,
    landing_move_speed: f32,
    attack2_step: f32,

    krs: KinematicRigidSprite,
    animator: FrameAnimator,
}

impl Knight {
    pub fn new(frame_atlas: FrameAtlas, position: Vec2<f32>) -> Self {
        Self {
            curr_state: State::Idle,
            next_state: State::Idle,
            can_perform_combo: false,
            is_attack2_step_done: false,
            move_speed: 100.0,
            jump_speed: 150.0,
            landing_move_speed: 70.0,
            attack2_step: 8.0,
            krs: KinematicRigidSprite::new(position),
            animator: FrameAnimator::new(frame_atlas),
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        gravity: f32,
        rigid_colliders: &[Rect],
        input: &mut Input,
        renderer: &mut Renderer,
    ) {
        use sdl2::keyboard::Keycode::*;
        use State::*;

        let is_attack_action = input.key_is_pressed(Space);
        let is_left_action = input.key_is_down(A);
        let is_right_action = input.key_is_down(D);
        let is_jump_action = input.key_is_pressed(W);
        let is_roll_action = input.key_is_pressed(LCtrl);
        let is_step_action = is_right_action || is_left_action;
        let dir = if is_right_action { 1.0 } else { -1.0 };

        if self.krs.velocity.y < 0.0 {
            self.set_curr_state(JumpDown);
        }

        match self.curr_state {
            Idle => {
                self.set_curr_state(Idle);

                if is_attack_action {
                    self.set_curr_state(Attack0);
                    self.can_perform_combo = true;
                } else if is_jump_action {
                    self.set_curr_state(JumpUp);
                    self.krs.velocity.y += self.jump_speed;
                } else if is_left_action || is_right_action {
                    self.set_curr_state(Run);
                }
            }
            Attack0 => {
                self.is_attack2_step_done = false;
                if is_attack_action && self.can_perform_combo {
                    if self.animator.cycle > 0.7 {
                        self.set_next_state(Attack1);
                    } else {
                        self.can_perform_combo = false
                    }
                }
            }
            Attack1 => {
                if is_attack_action && self.can_perform_combo {
                    if self.animator.cycle > 0.7 {
                        self.set_next_state(Attack2);
                    } else {
                        self.can_perform_combo = false
                    }
                }
            }
            Attack2 => {
                if !self.is_attack2_step_done {
                    self.krs.position.x +=
                        self.krs.look_dir * self.attack2_step;
                    self.is_attack2_step_done = true;
                }
            }
            Run => {
                if is_roll_action {
                    self.set_curr_state(Roll);
                } else if is_jump_action {
                    self.set_curr_state(JumpUp);
                    self.krs.velocity.y += self.jump_speed;
                } else if is_attack_action {
                    self.set_curr_state(Attack0);
                    self.can_perform_combo = true;
                } else if is_step_action {
                    self.krs.do_immediate_step(dt, self.move_speed, dir);
                } else {
                    self.set_curr_state(Idle);
                }
            }
            Roll => {
                let speed = 150.0 * (1.0 - self.animator.cycle.powf(2.0));
                self.krs.position.x += self.krs.look_dir * speed * dt;
            }
            JumpUp => {
                if self.krs.velocity.y > 0.0 {
                    self.set_next_state(JumpUp);
                } else {
                    self.set_next_state(JumpDown);
                }

                if is_step_action {
                    self.krs.do_immediate_step(dt, self.move_speed, dir);
                }
            }
            JumpDown => {
                if self.krs.is_grounded {
                    self.set_next_state(JumpLanding);
                } else {
                    self.set_next_state(JumpDown);
                }

                if is_step_action {
                    self.krs.do_immediate_step(dt, self.move_speed, dir);
                }
            }
            JumpLanding => {
                if is_step_action {
                    self.krs.do_immediate_step(
                        dt,
                        self.landing_move_speed,
                        dir,
                    );
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

        if self.curr_state != state {
            self.next_state = Idle;
        }

        self.curr_state = state;
        match self.curr_state {
            Idle => self.animator.play("knight_idle", 0.07, true),
            Run => self.animator.play("knight_run", 0.07, true),
            Roll => self.animator.play("knight_roll", 0.07, false),
            JumpUp => self.animator.play("knight_jump_up", 0.07, false),
            JumpDown => {
                self.animator.play("knight_jump_down", 0.07, false)
            }
            JumpLanding => {
                self.animator.play("knight_jump_landing", 0.07, false)
            }
            Attack0 => self.animator.play("knight_attack_0", 0.07, false),
            Attack1 => self.animator.play("knight_attack_1", 0.07, false),
            Attack2 => self.animator.play("knight_attack_2", 0.07, false),
        }
    }

    fn set_next_state(&mut self, state: State) {
        self.next_state = state;
    }

    pub fn get_position(&self) -> Vec2<f32> {
        self.krs.position
    }
}
