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
        let sprite_xywh =
            self.frame_atlas.get_sprite_xywh("knight_attack", 7);
        let rect_size =
            Vec2::new(sprite_xywh[2], sprite_xywh[3]).scale(3.0);
        let rect = Rect::from_bot_center(Vec2::zeros(), rect_size);
        let primitive = DrawPrimitive {
            z: 0.0,
            rect,
            space: SpaceType::WorldSpace,
            tex: TextureType::SpriteTexture,
            xywh: sprite_xywh,
            rgba: [0.3, 0.0, 0.0, 1.0],
            effect: 0,
            flip: false,
        };
        renderer.push_primitive(primitive);
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
