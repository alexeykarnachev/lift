use crate::frame::*;
use crate::renderer::*;
use crate::vec::*;

pub struct KinematicRigidSprite {
    pub position: Vec2<f32>,
    pub velocity: Vec2<f32>,
    pub is_grounded: bool,
    pub look_dir: f32,
}

impl KinematicRigidSprite {
    pub fn new(position: Vec2<f32>) -> Self {
        Self {
            position,
            velocity: Vec2::zeros(),
            is_grounded: false,
            look_dir: 1.0,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        gravity: f32,
        frame: Frame,
        rigid_colliders: &[Rect],
        renderer: &mut Renderer,
    ) {
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
    }

    pub fn do_immediate_step(&mut self, dt: f32, speed: f32, dir: f32) {
        self.look_dir = dir;
        self.position.x += dt * speed * dir;
    }
}
