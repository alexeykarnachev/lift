use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum State {
    Idle,
    Move,
    Roll,
    Attack0,
    Attack1,
    Attack2,
}

pub struct Knight {
    curr_state: State,
    next_state: State,
    can_perform_combo: bool,

    position: Vec2<f32>,
    look_at_right: bool,

    animator: FrameAnimator,
}

impl Knight {
    pub fn new(frame_atlas: FrameAtlas, position: Vec2<f32>) -> Self {
        use State::*;
        let animator =
            FrameAnimator::new(frame_atlas, "knight_idle", 0.1, true);

        Self {
            curr_state: Idle,
            next_state: Idle,
            can_perform_combo: false,
            position,
            look_at_right: true,
            animator,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        input: &mut Input,
        renderer: &mut Renderer,
    ) {
        use sdl2::keyboard::Keycode::*;
        use State::*;
        let is_attack_action = input.key_is_pressed(Space);
        let is_left_action = input.key_is_down(A);
        let is_right_action = input.key_is_down(D);
        let is_roll_action = input.key_is_pressed(LCtrl);

        match self.curr_state {
            Idle => {
                if is_attack_action {
                    self.curr_state = Attack0;
                    self.next_state = Idle;
                    self.can_perform_combo = true;
                } else if is_left_action || is_right_action {
                    self.curr_state = Move;
                }
            }
            Attack0 => {
                if is_attack_action && self.can_perform_combo {
                    if self.animator.cycle > 0.7 {
                        self.next_state = Attack1;
                    } else {
                        self.next_state = Idle;
                        self.can_perform_combo = false
                    }
                }
            }
            Attack1 => {
                if is_attack_action && self.can_perform_combo {
                    if self.animator.cycle > 0.7 {
                        self.next_state = Attack2;
                    } else {
                        self.next_state = Idle;
                        self.can_perform_combo = false
                    }
                }
            }
            Attack2 => {
                self.next_state = Idle;
            }
            Move => {
                if is_roll_action {
                    self.curr_state = Roll;
                    self.next_state = Idle;
                } else if is_attack_action {
                    self.curr_state = Attack0;
                    self.next_state = Idle;
                    self.can_perform_combo = true;
                } else if is_left_action {
                    self.position.x -= 100.0 * dt;
                    self.look_at_right = false;
                } else if is_right_action {
                    self.position.x += 100.0 * dt;
                    self.look_at_right = true;
                } else {
                    self.curr_state = Idle;
                }
            }
            Roll => {
                let speed = 150.0 * (1.0 - self.animator.cycle.powf(2.0));
                if self.look_at_right {
                    self.position.x += speed * dt;
                } else {
                    self.position.x -= speed * dt;
                }
            }
        }

        self.play_animation();

        let frame = self.animator.update(dt);
        let primitive = DrawPrimitive::world_sprite(
            frame.sprite,
            Pivot::BotCenter(self.position),
            false,
            !self.look_at_right,
        );
        renderer.push_primitive(primitive);

        if self.animator.is_finished() {
            self.curr_state = self.next_state;
            self.next_state = Idle;
        }
    }

    fn play_animation(&mut self) {
        use State::*;
        match self.curr_state {
            Idle => self.animator.play("knight_idle", 0.07, true),
            Move => self.animator.play("knight_walk", 0.07, true),
            Roll => self.animator.play("knight_roll", 0.07, false),
            Attack0 => self.animator.play("knight_attack_0", 0.07, false),
            Attack1 => self.animator.play("knight_attack_1", 0.07, false),
            Attack2 => self.animator.play("knight_attack_2", 0.07, false),
        }
    }
}
