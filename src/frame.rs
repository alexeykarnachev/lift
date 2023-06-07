use crate::vec::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct XYWH {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl XYWH {
    pub fn zeros() -> Self {
        Self {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.x as f32, self.y as f32, self.w as f32, self.h as f32]
    }

    pub fn to_position(&self) -> Vec2<f32> {
        Vec2::new(self.x as f32, self.y as f32)
    }

    pub fn to_size(&self) -> Vec2<f32> {
        Vec2::new(self.w as f32, self.h as f32)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Frame {
    pub sprite: XYWH,
    masks: HashMap<String, XYWH>,
}

impl Frame {
    pub fn get_mask(
        &self,
        name: &str,
        pivot: Pivot,
        flip: bool,
    ) -> Option<Rect> {
        use Pivot::*;

        if let Some(xywh) = self.masks.get(name) {
            let size = xywh.to_size();
            let position = xywh.to_position();
            let pivot = match pivot {
                BotLeft(pivot) => pivot,
                BotCenter(pivot) => {
                    pivot.add_x(-0.5 * self.sprite.w as f32)
                }
                _ => {
                    todo!()
                }
            };

            let rect = if flip {
                let pivot = pivot.add_x(self.sprite.w as f32);
                let position = pivot + position.mul_x(-1.0);
                Rect::from_top_right(position, size)
            } else {
                let position = pivot + position;
                Rect::from_top_left(position, size)
            };

            return Some(rect);
        }

        None
    }
}

#[derive(Deserialize, Clone)]
pub struct FrameAtlas {
    size: [u32; 2],
    #[serde(rename = "frames")]
    name_to_frames: HashMap<String, Vec<Frame>>,
}

impl FrameAtlas {
    pub fn new(file_path: &str) -> Self {
        let meta = fs::read_to_string(file_path).unwrap();

        serde_json::from_str(&meta).unwrap()
    }

    pub fn new_animator(&'static self) -> FrameAnimator {
        FrameAnimator::new(self)
    }
}

#[derive(Clone, Copy)]
pub struct FrameAnimator {
    atlas: &'static FrameAtlas,
    is_started: bool,

    name: &'static str,
    frame_duration: f32,
    is_repeat: bool,

    pub progress: f32,
}

impl FrameAnimator {
    pub fn new(atlas: &'static FrameAtlas) -> Self {
        Self {
            atlas,
            is_started: false,
            name: "",
            frame_duration: 0.0,
            is_repeat: false,
            progress: 0.0,
        }
    }

    pub fn play(
        &mut self,
        name: &'static str,
        frame_duration: f32,
        is_repeat: bool,
    ) {
        self.is_started = true;

        if name != self.name
            || frame_duration != self.frame_duration
            || is_repeat != self.is_repeat
        {
            self.name = name;
            self.frame_duration = frame_duration;
            self.is_repeat = is_repeat;
            self.progress = 0.0;
        }
    }

    pub fn is_finished(&self) -> bool {
        !self.is_repeat && self.progress == 1.0
    }

    pub fn update(&mut self, dt: f32) -> Option<&Frame> {
        if !self.is_started {
            return None;
        }

        let frames = self
            .atlas
            .name_to_frames
            .get(self.name)
            .expect(&format!("FrameAtlas should have {}", self.name));
        let n_frames = frames.len() as f32;
        let max_idx = n_frames - 1.0;

        self.progress += dt / (n_frames * self.frame_duration);
        if self.is_repeat {
            self.progress -= self.progress.floor();
        } else {
            self.progress = self.progress.min(1.0);
        };

        let idx = (self.progress * max_idx).round() as usize;

        Some(&frames[idx])
    }
}
