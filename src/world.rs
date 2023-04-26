#![allow(dead_code)]
#![allow(unused_variables)]

use crate::input::Input;
use crate::vec::Vec2;

pub struct World {
    pub lift: Lift,
    pub camera: Camera,
}

impl World {
    pub fn update(&mut self, dt: f32, input: &Input) {
        self.lift.update(dt);
        self.camera.update(dt, input);
    }
}

pub struct Lift {
    pub size: Vec2<f32>,

    pub elevation: f32,
    pub target_elevation: f32,

    pub max_speed: f32,
}

impl Default for Lift {
    fn default() -> Self {
        Self {
            size: Vec2::new(1.0, 2.0),
            elevation: 0.0,
            target_elevation: 0.0,
            max_speed: 10.0,
        }
    }
}

impl Lift {
    pub fn update(&mut self, dt: f32) {}
}

pub struct Camera {
    pub position: Vec2<f32>,
    pub orientation: f32,

    pub view_width: f32,
    pub aspect: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            orientation: 0.0,
            view_width: 10.0,
            aspect: 1.77,
        }
    }
}

impl Camera {
    pub fn update(&mut self, dt: f32, input: &Input) {
        self.aspect =
            input.window_size.x as f32 / input.window_size.y as f32;
        if input.wheel_d != 0 {
            let diff = self.view_width * 0.1 * input.wheel_d as f32;
            self.view_width -= diff;
        }

        if input.mmb_is_down {
            let cursor_world_pos =
                screen_to_world(self, input.window_size, input.cursor_pos);
            let cursor_world_prev_pos = screen_to_world(
                self,
                input.window_size,
                input.cursor_prev_pos,
            );
            let mut cursor_world_diff =
                cursor_world_pos - cursor_world_prev_pos;
            cursor_world_diff
                .rotate_inplace(Vec2::new(0.0, 0.0), -self.orientation);
            self.position -= cursor_world_diff;
        }
    }

    pub fn get_view_size(&self) -> Vec2<f32> {
        let view_height = self.view_width / self.aspect;

        Vec2::new(self.view_width, view_height)
    }
}

fn screen_to_world(
    camera: &Camera,
    window_size: Vec2<i32>,
    screen_pos: Vec2<i32>,
) -> Vec2<f32> {
    let width = window_size.x as f32;
    let height = window_size.y as f32;
    let window_size = Vec2::new(width, height);
    let aspect = width / height;

    let view_size = camera.get_view_size();

    let screen_pos = Vec2::<f32>::new(
        screen_pos.x as f32,
        height - screen_pos.y as f32,
    );
    let bot_left = camera.position - view_size.scale(0.5);
    let mut world_pos = bot_left + view_size * screen_pos / window_size;
    world_pos = world_pos.rotate(Vec2::zeros(), camera.orientation);
    return world_pos;
}
