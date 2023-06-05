use crate::entities::knight::Knight;
use crate::frame::FrameAtlas;
// use crate::input::Keyaction;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;

pub struct Game {
    camera: Camera,
    rigid_colliders: Vec<Rect>,

    player: Knight,

    gravity: f32,
}

impl Game {
    pub fn new(frame_atlas_fp: &str) -> Self {
        let frame_atlas = FrameAtlas::new(frame_atlas_fp);
        let camera = Camera::new(Vec2::zeros());
        let player = Knight::new(frame_atlas, Vec2::zeros());
        let rigid_colliders = vec![
            Rect::from_top_center(
                Vec2::new(0.0, -20.0),
                Vec2::new(1000.0, 50.0),
            ),
            Rect::from_bot_center(
                Vec2::new(-50.0, -20.0),
                Vec2::new(25.0, 100.0),
            ),
        ];

        Self {
            camera,
            rigid_colliders,
            player,
            gravity: 200.0,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        renderer: &mut Renderer,
        input: &mut Input,
    ) {
        renderer
            .set_camera(self.camera.position, self.camera.get_view_size());

        self.player.update(
            dt,
            self.gravity,
            &self.rigid_colliders,
            input,
            renderer,
        );
        for rect in self.rigid_colliders.iter() {
            let primitive =
                DrawPrimitive::world_rect(*rect, Color::red(0.5));
            renderer.push_primitive(primitive);
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
