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

    position: Vec2<f32>,
    velocity: Vec2<f32>,
    look_dir: f32,
    is_grounded: bool,

    move_speed: f32,

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
            move_speed: 100.0,
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

        self.curr_state = state;
        match self.curr_state {
            Idle => self.animator.play("wolf_idle", 0.07, true),
        }
    }

    fn set_next_state(&mut self, state: State) {
        self.next_state = state;
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
