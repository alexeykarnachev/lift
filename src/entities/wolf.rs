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

    position: Vec2<f32>,
    velocity: Vec2<f32>,
    look_dir: f32,
    is_grounded: bool,

    move_speed: f32,
    view_dist: f32,
    attack_dist: f32,

    animator: FrameAnimator,
}

impl Wolf {
    pub fn new(frame_atlas: FrameAtlas, position: Vec2<f32>) -> Self {
        Self {
            curr_state: State::Idle,
            next_state: State::Idle,
            position,
            velocity: Vec2::zeros(),
            look_dir: 1.0,
            is_grounded: false,
            move_speed: 50.0,
            view_dist: 200.0,
            attack_dist: 35.0,
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

        let to_player = player_position - self.position;
        let dist = to_player.len();
        let dir = to_player.x.signum();

        if dist <= self.view_dist {
            self.look_dir = dir;
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
                    self.do_immediate_step(dt, self.move_speed, dir);
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

        self.velocity.y -= gravity * dt;
        self.position += self.velocity.scale(dt);

        let primitive = DrawPrimitive::world_sprite(
            frame.sprite,
            Pivot::BotCenter(self.position),
            false,
            self.look_dir < 0.0,
        );
        renderer.push_primitive(primitive);

        if let Some(my_collider) = frame.get_mask(
            "rigid",
            Pivot::BotCenter(self.position),
            self.look_dir < 0.0,
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
            renderer.push_primitive(DrawPrimitive::world_rect(
                my_collider,
                Color::green(0.5),
            ));
        }

        if let Some(attack) = frame.get_mask(
            "attack",
            Pivot::BotCenter(self.position),
            self.look_dir < 0.0,
        ) {
            renderer.push_primitive(DrawPrimitive::world_rect(
                attack,
                Color::red(0.5),
            ));
        }
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

    pub fn do_immediate_step(&mut self, dt: f32, speed: f32, dir: f32) {
        self.look_dir = dir;
        self.position.x += dt * speed * dir;
    }
}
