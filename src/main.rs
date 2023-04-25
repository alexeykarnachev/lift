#![allow(dead_code)]
#![allow(unused_variables)]

use renderer::Renderer;
use std::time::Instant;
use world::{Camera, Lift, World};

mod renderer;
mod world;

fn main() {
    let sdl = sdl2::init().unwrap();
    let renderer = Renderer::create(&sdl, "Lift", 800, 600);

    // -------------------------------------------------------------------
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

    let mut world = World { lift, camera };

    // -------------------------------------------------------------------
    let mut event_pump = sdl.event_pump().unwrap();
    let mut prev_upd_time = Instant::now();
    'main: loop {
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                break 'main;
            }
        }

        let dt = prev_upd_time.elapsed().as_nanos() as f32 / 1.0e9;
        println!("{}", dt);
        world.update(dt);
        prev_upd_time = Instant::now();

        renderer.render(&world);
    }
}
