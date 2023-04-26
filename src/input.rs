use crate::vec::Vec2;
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;

struct Accum {
    cursor_pos: Vec2<i32>,
    lmb_press_pos: Option<Vec2<i32>>,
    rmb_press_pos: Option<Vec2<i32>>,
    wheel_d: i32,
}

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

    pub wheel_d: i32,
}

impl Input {
    pub fn create(initial_window_size: Vec2<u32>) -> Self {
        Self {
            accum: Accum {
                cursor_pos: Vec2::new(0, 0),
                lmb_press_pos: None,
                rmb_press_pos: None,
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
            wheel_d: 0,
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
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Right,
                ..
            } => {
                self.rmb_is_down = false;
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
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.cursor_d = self.accum.cursor_pos - self.cursor_pos;
        self.cursor_prev_pos = self.cursor_pos;
        self.cursor_pos = self.accum.cursor_pos;
        self.lmb_press_pos = self.accum.lmb_press_pos;
        self.rmb_press_pos = self.accum.rmb_press_pos;
        self.wheel_d = self.accum.wheel_d;

        self.accum.lmb_press_pos = None;
        self.accum.rmb_press_pos = None;
        self.accum.wheel_d = 0;
    }
}
