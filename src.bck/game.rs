use crate::frame_atlas::FrameAtlas;
use crate::renderer::*;
use crate::vec::*;
use crate::Input;

pub struct Game {
    frame_atlas: FrameAtlas,

    camera: Camera,
}

impl Game {
    pub fn new(frame_atlas_fp: &str) -> Self {
        let frame_atlas = FrameAtlas::new(frame_atlas_fp);
        let camera = Camera::new(Vec2::zeros());

        Self {
            frame_atlas,
            camera,
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

        let idx = 7;

        let xywh = self.frame_atlas.get_sprite_xywh("knight_attack", idx);
        let pivot = Pivot::BotCenter(Vec2::zeros());
        let primitive =
            DrawPrimitive::world_sprite(xywh, pivot, false, false);
        let bl = primitive.rect.get_bot_left();
        renderer.push_primitive(primitive);

        if let Some(rect) =
            self.frame_atlas.get_attack_collider("knight_attack", idx)
        {
            let rect = rect.translate(bl);
            let primitive = DrawPrimitive::world_rect(rect, Color::green(0.5));
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
            // view_width: 500.0,
            view_width: 150.0,
            aspect: 1.77,
        }
    }

    pub fn get_view_size(&self) -> Vec2<f32> {
        let view_height = self.view_width / self.aspect;

        Vec2::new(self.view_width, view_height)
    }
}
