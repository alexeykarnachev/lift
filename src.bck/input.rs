use crate::vec::Vec2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

#[derive(Debug)]
pub enum Keyaction {
    Left,
    Right,
    Up,
    Down,
    Attack,
    Dash,
    SkillsTree,
    _N,
}

const N_KEYACTIONS: usize = Keyaction::_N as usize;

pub fn keycode_to_keyaction(
    code: Keycode,
    repeat: bool,
) -> Option<Keyaction> {
    match code {
        Keycode::Left | Keycode::A => Some(Keyaction::Left),
        Keycode::Right | Keycode::D => Some(Keyaction::Right),
        Keycode::Up | Keycode::W => Some(Keyaction::Up),
        Keycode::Down | Keycode::S => Some(Keyaction::Down),
        Keycode::Space => Some(Keyaction::Attack),
        Keycode::LCtrl => Some(Keyaction::Dash),
        Keycode::T if !repeat => Some(Keyaction::SkillsTree),
        _ => None,
    }
}

struct Accum {
    cursor_pos: Vec2<i32>,
    lmb_press_pos: Option<Vec2<i32>>,
    rmb_press_pos: Option<Vec2<i32>>,
    lmb_is_just_up: bool,
    rmb_is_just_up: bool,
    wheel_d: i32,
}

const N_KEYS: usize = 256;
pub struct Input {
    accum: Accum,

    pub should_quit: bool,
    pub window_size: Vec2<i32>,

    pub cursor_pos: Vec2<i32>,
    pub cursor_prev_pos: Vec2<i32>,
    pub cursor_d: Vec2<i32>,

    pub lmb_press_pos: Option<Vec2<i32>>,
    pub rmb_press_pos: Option<Vec2<i32>>,
    pub lmb_is_down: bool,
    pub rmb_is_down: bool,
    pub mmb_is_down: bool,
    pub lmb_is_just_up: bool,
    pub rmb_is_just_up: bool,

    pub wheel_d: i32,

    pub key_is_down: [bool; N_KEYACTIONS],
}

impl Input {
    pub fn new(initial_window_size: Vec2<u32>) -> Self {
        Self {
            accum: Accum {
                cursor_pos: Vec2::new(0, 0),
                lmb_press_pos: None,
                rmb_press_pos: None,
                lmb_is_just_up: false,
                rmb_is_just_up: false,
                wheel_d: 0,
            },
            should_quit: false,
            window_size: Vec2::new(
                initial_window_size.x as i32,
                initial_window_size.y as i32,
            ),
            cursor_pos: Vec2::new(0, 0),
            cursor_prev_pos: Vec2::new(0, 0),
            cursor_d: Vec2::new(0, 0),
            lmb_press_pos: None,
            rmb_press_pos: None,
            lmb_is_down: false,
            rmb_is_down: false,
            mmb_is_down: false,
            lmb_is_just_up: false,
            rmb_is_just_up: false,
            wheel_d: 0,
            key_is_down: [false; N_KEYACTIONS],
        }
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
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                if !self.lmb_is_down {
                    self.accum.lmb_press_pos = Some(Vec2::new(*x, *y));
                }

                self.lmb_is_down = true;
            }
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Right,
                x,
                y,
                ..
            } => {
                if !self.rmb_is_down {
                    self.accum.rmb_press_pos = Some(Vec2::new(*x, *y));
                }

                self.rmb_is_down = true;
            }
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Middle,
                ..
            } => {
                self.mmb_is_down = true;
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                self.lmb_is_down = false;
                self.accum.lmb_is_just_up = true;
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Right,
                ..
            } => {
                self.rmb_is_down = false;
                self.accum.rmb_is_just_up = true;
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Middle,
                ..
            } => {
                self.mmb_is_down = false;
            }
            Event::MouseWheel { y, .. } => {
                self.accum.wheel_d += *y;
            }
            Event::KeyDown {
                keycode: Some(code),
                repeat,
                ..
            } => {
                if let Some(action) = keycode_to_keyaction(*code, *repeat)
                {
                    self.key_is_down[action as usize] = true;
                }
            }
            Event::KeyUp {
                keycode: Some(code),
                ..
            } => {
                if let Some(action) = keycode_to_keyaction(*code, false) {
                    self.key_is_down[action as usize] = false;
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.cursor_d = self.accum.cursor_pos - self.cursor_pos;
        self.cursor_prev_pos = self.cursor_pos;
        self.cursor_pos = self.accum.cursor_pos;
        self.lmb_press_pos = self.accum.lmb_press_pos;
        self.rmb_press_pos = self.accum.rmb_press_pos;
        self.lmb_is_just_up = self.accum.lmb_is_just_up;
        self.rmb_is_just_up = self.accum.rmb_is_just_up;
        self.wheel_d = self.accum.wheel_d;

        self.accum.lmb_press_pos = None;
        self.accum.rmb_press_pos = None;
        self.accum.lmb_is_just_up = false;
        self.accum.rmb_is_just_up = false;
        self.accum.wheel_d = 0;
    }

    pub fn is_action(&self, action: Keyaction) -> bool {
        self.key_is_down[action as usize]
    }

    pub fn take_action(&mut self, action: Keyaction) -> bool {
        let idx = action as usize;
        let is_action = self.key_is_down[idx];
        self.key_is_down[idx] = false;

        is_action
    }
}
