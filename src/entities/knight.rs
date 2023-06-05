use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
enum State {
    Idle,
    Walk,
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

    position: Vec2<f32>,
    velocity: Vec2<f32>,
    look_dir: f32,
    is_grounded: bool,

    move_speed: f32,
    jump_speed: f32,
    landing_move_speed_mult: f32,
    attack2_step: f32,

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
            is_attack2_step_done: false,
            position,
            velocity: Vec2::zeros(),
            look_dir: 1.0,
            is_grounded: false,
            move_speed: 100.0,
            jump_speed: 150.0,
            landing_move_speed_mult: 0.7,
            attack2_step: 8.0,
            animator,
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

        match self.curr_state {
            Idle => {
                if is_attack_action {
                    self.set_curr_state(Attack0);
                    self.can_perform_combo = true;
                } else if is_jump_action {
                    self.set_curr_state(JumpUp);
                    self.velocity.y += self.jump_speed;
                } else if is_left_action || is_right_action {
                    self.set_curr_state(Walk);
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
                    self.position.x += self.look_dir * self.attack2_step;
                    self.is_attack2_step_done = true;
                }
            }
            Walk => {
                if is_roll_action {
                    self.set_curr_state(Roll);
                } else if is_jump_action {
                    self.set_curr_state(JumpUp);
                    self.velocity.y += self.jump_speed;
                } else if is_attack_action {
                    self.set_curr_state(Attack0);
                    self.can_perform_combo = true;
                } else if is_step_action {
                    self.immediate_step(dt, is_right_action);
                } else {
                    self.set_curr_state(Idle);
                }
            }
            Roll => {
                let speed = 150.0 * (1.0 - self.animator.cycle.powf(2.0));
                self.position.x += self.look_dir * speed * dt;
            }
            JumpUp => {
                if self.velocity.y > 0.0 {
                    self.set_next_state(JumpUp);
                } else {
                    self.set_next_state(JumpDown);
                }

                if is_step_action {
                    self.immediate_step(dt, is_right_action);
                }
            }
            JumpDown => {
                if self.is_grounded {
                    self.set_next_state(JumpLanding);
                } else {
                    self.set_next_state(JumpDown);
                }

                if is_step_action {
                    self.immediate_step(dt, is_right_action);
                }
            }
            JumpLanding => {
                if is_roll_action {
                    self.set_curr_state(Roll);
                } else if is_step_action {
                    self.immediate_step(
                        self.landing_move_speed_mult * dt,
                        is_right_action,
                    );
                }
            }
        }

        self.velocity.y -= gravity * dt;
        self.position += self.velocity.scale(dt);

        let frame = self.animator.update(dt);
        let primitive = DrawPrimitive::world_sprite(
            frame.sprite,
            Pivot::BotCenter(self.position),
            false,
            self.look_dir < 0.0,
        );
        let sprite_rect = primitive.rect;
        renderer.push_primitive(primitive);

        if self.animator.is_finished() {
            self.set_curr_state(self.next_state);
        }

        if let Some(my_collider) = frame.get_mask(
            "rigid",
            Pivot::BotLeft(sprite_rect.get_bot_left()),
            self.look_dir < 0.0,
        ) {
            let primitive =
                DrawPrimitive::world_rect(my_collider, Color::green(0.5));
            // renderer.push_primitive(primitive);
            self.resolve_collisions(my_collider, rigid_colliders);
        }
    }

    fn set_curr_state(&mut self, state: State) {
        use State::*;

        if self.curr_state != state {
            self.next_state = Idle;
        }

        self.curr_state = state;
        match self.curr_state {
            Idle => self.animator.play("knight_idle", 0.07, true),
            Walk => self.animator.play("knight_walk", 0.07, true),
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

    fn immediate_step(&mut self, dt: f32, is_right: bool) {
        self.look_dir = if is_right { 1.0 } else { -1.0 };

        self.position.x += self.look_dir * dt * self.move_speed;
    }

    fn resolve_collisions(
        &mut self,
        my_collider: Rect,
        rigid_colliders: &[Rect],
    ) {
        let mut is_grounded = false;
        for collider in rigid_colliders {
            let mtv = my_collider.collide_aabb(*collider);
            self.position += mtv;

            if mtv.y.abs() > 0.0 {
                self.velocity.y = 0.0;
            }

            if mtv.x.abs() > 0.0 {
                self.velocity.x = 0.0;
            }

            is_grounded |= mtv.y > 0.0;
        }

        self.is_grounded = is_grounded;
    }
}
