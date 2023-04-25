#![allow(dead_code)]
#![allow(unused_variables)]

use crate::world::World;

pub struct Renderer {}

impl Renderer {
    pub fn create() -> Self {
        Self {}
    }

    pub fn render(&self, world: &World) {}
}
