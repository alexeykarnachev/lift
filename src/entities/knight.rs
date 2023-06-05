use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
enum State {
    Idle,
    Walk,
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
    velocity: Vec2<f32>,
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
            velocity: Vec2::zeros(),
            look_at_right: true,
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
        let is_roll_action = input.key_is_pressed(LCtrl);

        match self.curr_state {
            Idle => {
                if is_attack_action {
                    self.set_curr_state(Attack0);
                    self.can_perform_combo = true;
                } else if is_left_action || is_right_action {
                    self.set_curr_state(Walk);
                }
            }
            Attack0 => {
                if is_attack_action && self.can_perform_combo {
                    if self.animator.cycle > 0.7 {
                        self.set_next_state(Attack1);
                    } else {
                        self.set_next_state(Idle);
                        self.can_perform_combo = false
                    }
                }
            }
            Attack1 => {
                if is_attack_action && self.can_perform_combo {
                    if self.animator.cycle > 0.7 {
                        self.set_next_state(Attack2);
                    } else {
                        self.set_next_state(Idle);
                        self.can_perform_combo = false
                    }
                }
            }
            Attack2 => {}
            Walk => {
                if is_roll_action {
                    self.set_curr_state(Roll);
                } else if is_attack_action {
                    self.set_curr_state(Attack0);
                    self.can_perform_combo = true;
                } else if is_left_action {
                    self.position.x -= 100.0 * dt;
                    self.look_at_right = false;
                } else if is_right_action {
                    self.position.x += 100.0 * dt;
                    self.look_at_right = true;
                } else {
                    self.set_curr_state(Idle);
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

        let frame = self.animator.update(dt);
        let primitive = DrawPrimitive::world_sprite(
            frame.sprite,
            Pivot::BotCenter(self.position),
            false,
            !self.look_at_right,
        );
        let sprite_rect = primitive.rect;
        renderer.push_primitive(primitive);

        if self.animator.is_finished() {
            self.set_curr_state(self.next_state);
        }

        if let Some(my_collider) = frame.get_mask(
            "rigid",
            Pivot::BotLeft(sprite_rect.get_bot_left()),
            !self.look_at_right,
        ) {
            let primitive =
                DrawPrimitive::world_rect(my_collider, Color::green(0.5));
            renderer.push_primitive(primitive);
            self.update_kinematic(
                dt,
                gravity,
                my_collider,
                rigid_colliders,
            );
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
            Attack0 => self.animator.play("knight_attack_0", 0.07, false),
            Attack1 => self.animator.play("knight_attack_1", 0.07, false),
            Attack2 => self.animator.play("knight_attack_2", 0.07, false),
        }
    }

    fn set_next_state(&mut self, state: State) {
        self.next_state = state;
    }

    fn update_kinematic(
        &mut self,
        dt: f32,
        gravity: f32,
        my_collider: Rect,
        rigid_colliders: &[Rect],
    ) {
        self.velocity.y -= gravity * dt;

        let step = self.velocity.scale(dt);
        self.position += step;

        for collider in rigid_colliders {
            let mtv = my_collider.collide_aabb(*collider);
            self.position += mtv;

            if mtv.y.abs() > 0.0 {
                self.velocity.y = 0.0
            }
            if mtv.x.abs() > 0.0 {
                self.velocity.x = 0.0
            }
        }
    }
}
