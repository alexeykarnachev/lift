#![allow(dead_code)]
#![allow(unused_variables)]

use renderer::Renderer;
use sdlgl::SDLGL;
use std::time::Instant;
use world::{Camera, Lift, World};

mod renderer;
mod sdlgl;
mod world;

fn main() {
    // let gl_attr = video.gl_attr();
    // gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    // gl_attr.set_context_version(4, 6);

    let sdlgl = SDLGL::create("Lift", 800, 600);

    let renderer = Renderer::create();

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
    let mut event_pump = sdlgl.sdl.event_pump().unwrap();
    let mut prev_upd_time = Instant::now();
    'main: loop {
        for event in event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event {
                break 'main;
            }
        }

        let dt = prev_upd_time.elapsed().as_nanos() as f32 / 1.0e9;

        world.update(dt);
        renderer.render(&world);

        prev_upd_time = Instant::now();
    }
}
