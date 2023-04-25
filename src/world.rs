#![allow(dead_code)]
#![allow(unused_variables)]

pub struct World {
    pub lift: Lift,
    pub camera: Camera,
}

impl World {
    pub fn update(&mut self, dt: f32) {
        self.lift.update(dt);
        self.camera.update(dt);
    }
}

#[derive(Default)]
pub struct Lift {
    pub elevation: f32,
    pub width: f32,
    pub height: f32,

    pub target_elevation: f32,

    pub max_speed: f32,
}

impl Lift {
    pub fn update(&mut self, dt: f32) {}
}

#[derive(Default)]
pub struct Camera {
    pub x: f32,
    pub y: f32,

    pub view_height: f32,
}

impl Camera {
    pub fn update(&mut self, dt: f32) {}
}
