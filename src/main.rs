#![allow(dead_code)]
#![allow(unused_variables)]

use input::Input;
use renderer::Renderer;
use std::time::Instant;
use vec::Vec2;
use world::World;

mod input;
mod renderer;
mod vec;
mod world;

fn main() {
    let window_size: Vec2<u32> = Vec2::new(1600, 1000);
    let sdl = sdl2::init().unwrap();
    let mut renderer = Renderer::create(&sdl, "Lift", window_size);
    let mut input = Input::create(window_size);

    // ------------------------------------------------------------------
    let n_floors = 16;
    let floor_size = Vec2::new(100.0, 2.5);
    let mut world = World::create(n_floors, floor_size);

    // ------------------------------------------------------------------
    let mut event_pump = sdl.event_pump().unwrap();
    let mut prev_upd_time = Instant::now();
    'main: loop {
        for event in event_pump.poll_iter() {
            input.handle_event(&event);
            if input.should_quit {
                break 'main;
            }
        }
        input.update();

        let dt = prev_upd_time.elapsed().as_nanos() as f32 / 1.0e9;
        world.update(dt, &input);
        prev_upd_time = Instant::now();

        renderer.render(&world);
    }
}
