use crate::entities::knight::Knight;
use crate::frame::FrameAtlas;
// use crate::input::Keyaction;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;

pub struct Game {
    camera: Camera,

    player: Knight,
}

impl Game {
    pub fn new(frame_atlas_fp: &str) -> Self {
        let frame_atlas = FrameAtlas::new(frame_atlas_fp);
        let camera = Camera::new(Vec2::zeros());
        let player = Knight::new(frame_atlas, Vec2::zeros());

        Self { camera, player }
    }

    pub fn update(
        &mut self,
        dt: f32,
        renderer: &mut Renderer,
        input: &mut Input,
    ) {
        renderer
            .set_camera(self.camera.position, self.camera.get_view_size());

        self.player.update(dt, input, renderer);
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
