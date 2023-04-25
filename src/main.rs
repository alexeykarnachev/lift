#![allow(dead_code)]
#![allow(unused_variables)]

use std::time::Instant;
use world::{Lift, World, Camera};
use renderer::{Renderer};

mod world;
mod renderer;

fn main() {
    let lift = Lift {
        width: 1.0,
        height: 2.0,
        max_speed: 1.0,
        ..Default::default()
    };

    let camera = Camera {
        view_height: 10.0,
        ..Default::default()
    };

    let mut world = World { lift, camera};
    let renderer = Renderer::create();

    let mut prev_upd_time = Instant::now();
    loop {
        let dt = prev_upd_time.elapsed().as_nanos() as f32 / 1.0e9;

        world.update(dt);
        renderer.render(&world);

        prev_upd_time = Instant::now();
    }
}
