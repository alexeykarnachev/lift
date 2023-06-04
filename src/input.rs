use crate::vec::Vec2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;

pub fn keycode_as_usize(code: Keycode) -> usize {
    // CapsLock is the first key which has greater than ascii range
    // code number
    let caps_lock = Keycode::CapsLock as usize;

    let mut idx = code as usize;
    if idx >= Keycode::CapsLock as usize {
        idx = (idx - caps_lock) + 128
    }

    idx
}

struct Accum {
    cursor_pos: Vec2<i32>,

    mouse_press_pos: [Option<Vec2<i32>>; N_MOUSE_BUTTONS],
    mouse_is_pressed: [bool; N_MOUSE_BUTTONS],
    mouse_is_released: [bool; N_MOUSE_BUTTONS],

    key_is_pressed: [bool; N_KEYBOARD_KEYS],
    key_is_released: [bool; N_KEYBOARD_KEYS],

    wheel_d: i32,
}

impl Accum {
    pub fn new() -> Self {
        Self {
            cursor_pos: Vec2::new(0, 0),

            mouse_press_pos: [None; N_MOUSE_BUTTONS],
            mouse_is_pressed: [false; N_MOUSE_BUTTONS],
            mouse_is_released: [false; N_MOUSE_BUTTONS],

            key_is_pressed: [false; N_KEYBOARD_KEYS],
            key_is_released: [false; N_KEYBOARD_KEYS],

            wheel_d: 0,
        }
    }

    pub fn reset(&mut self) {
        self.mouse_press_pos.fill(None);
        self.mouse_is_pressed.fill(false);
        self.mouse_is_released.fill(false);
        self.key_is_pressed.fill(false);
        self.key_is_released.fill(false);
        self.wheel_d = 0;
    }
}

const N_KEYBOARD_KEYS: usize = 512;
const N_MOUSE_BUTTONS: usize = 16;
pub struct Input {
    accum: Accum,

    pub should_quit: bool,
    pub window_size: Vec2<i32>,

    pub cursor_pos: Vec2<i32>,
    pub cursor_prev_pos: Vec2<i32>,
    pub cursor_d: Vec2<i32>,

    mouse_press_pos: [Option<Vec2<i32>>; N_MOUSE_BUTTONS],
    mouse_is_down: [bool; N_MOUSE_BUTTONS],
    mouse_is_pressed: [bool; N_MOUSE_BUTTONS],
    mouse_is_released: [bool; N_MOUSE_BUTTONS],

    key_is_down: [bool; N_KEYBOARD_KEYS],
    key_is_pressed: [bool; N_KEYBOARD_KEYS],
    key_is_released: [bool; N_KEYBOARD_KEYS],

    pub wheel_d: i32,
}

impl Input {
    pub fn new(initial_window_size: Vec2<u32>) -> Self {
        Self {
            accum: Accum::new(),
            should_quit: false,
            window_size: Vec2::new(
                initial_window_size.x as i32,
                initial_window_size.y as i32,
            ),
            cursor_pos: Vec2::new(0, 0),
            cursor_prev_pos: Vec2::new(0, 0),
            cursor_d: Vec2::new(0, 0),
            mouse_press_pos: [None; N_MOUSE_BUTTONS],
            mouse_is_down: [false; N_MOUSE_BUTTONS],
            mouse_is_pressed: [false; N_MOUSE_BUTTONS],
            mouse_is_released: [false; N_MOUSE_BUTTONS],

            key_is_down: [false; N_KEYBOARD_KEYS],
            key_is_pressed: [false; N_KEYBOARD_KEYS],
            key_is_released: [false; N_KEYBOARD_KEYS],

            wheel_d: 0,
        }
    }

    pub fn key_is_down(&self, code: Keycode) -> bool {
        self.key_is_down[keycode_as_usize(code)]
    }

    pub fn key_is_pressed(&self, code: Keycode) -> bool {
        self.key_is_pressed[keycode_as_usize(code)]
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Quit { .. } => self.should_quit = true,
            Event::Window {
                win_event: WindowEvent::SizeChanged(w, h),
                ..
            } => {
                self.window_size = Vec2::new(*w, *h);
            }
            Event::MouseMotion { x, y, .. } => {
                self.accum.cursor_pos = Vec2::new(*x, *y);
            }
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                let idx = *mouse_btn as usize;
                if !self.mouse_is_down[idx] {
                    let press_pos = Some(Vec2::new(*x, *y));
                    self.accum.mouse_press_pos[idx] = press_pos;
                    self.accum.mouse_is_pressed[idx] = true;
                }

                self.mouse_is_down[idx] = true;
            }
            Event::MouseButtonUp { mouse_btn, .. } => {
                let idx = *mouse_btn as usize;
                self.mouse_is_down[idx] = false;
                self.accum.mouse_is_released[idx] = true;
            }
            Event::MouseWheel { y, .. } => {
                self.accum.wheel_d += *y;
            }
            Event::KeyDown {
                keycode: Some(code),
                repeat,
                ..
            } => {
                let idx = keycode_as_usize(*code);
                if !self.key_is_down[idx] {
                    self.accum.key_is_pressed[idx] = true;
                }

                self.key_is_down[idx] = true;
            }
            Event::KeyUp {
                keycode: Some(code),
                ..
            } => {
                let idx = keycode_as_usize(*code);
                self.accum.key_is_released[idx] = true;
                self.key_is_down[idx] = false;
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.cursor_d = self.accum.cursor_pos - self.cursor_pos;
        self.cursor_prev_pos = self.cursor_pos;
        self.cursor_pos = self.accum.cursor_pos;

        self.mouse_press_pos = self.accum.mouse_press_pos;
        self.mouse_is_pressed = self.accum.mouse_is_pressed;
        self.mouse_is_released = self.accum.mouse_is_released;
        self.key_is_pressed = self.accum.key_is_pressed;
        self.key_is_released = self.accum.key_is_released;

        self.wheel_d = self.accum.wheel_d;

        self.accum.reset();
    }
}
