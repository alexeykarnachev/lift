#![allow(dead_code)]
#![allow(unused_variables)]

use game::*;
use input::Input;
use renderer::Renderer;
use std::time::Instant;
use vec::Vec2;

mod entity;
mod frame_atlas;
mod game;
mod graphics;
mod gui;
mod input;
mod level;
mod player_stats;
mod prefabs;
mod renderer;
mod utils;
mod vec;

fn main() {
    // let window_size: Vec2<u32> = Vec2::new(2560, 1440);
    let window_size: Vec2<u32> = Vec2::new(1500, 800);
    let sdl = sdl2::init().unwrap();

    let mut renderer = Renderer::new(
        &sdl,
        "Lift",
        window_size,
        "./assets/sprites/atlas.png",
    );
    let mut input = Input::new(window_size);
    let mut game = Game::new("./assets/sprites/atlas.json");

    // ------------------------------------------------------------------
    let mut event_pump = sdl.event_pump().unwrap();
    let mut prev_upd_time = Instant::now();
    'main: loop {
        for event in event_pump.poll_iter() {
            input.handle_event(&event);
            if input.should_quit {
                // || game.state == GameState::Quit {
                break 'main;
            }
        }
        input.update();

        let dt = prev_upd_time.elapsed().as_nanos() as f32 / 1.0e9;
        renderer.clear_queue();
        game.update(dt, &mut renderer, &mut input);
        prev_upd_time = Instant::now();
        renderer.render();
    }
}
