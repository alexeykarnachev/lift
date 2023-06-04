use crate::vec::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Copy, Clone)]
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

#[derive(Deserialize, Clone)]
struct Frame {
    pub sprite: XYWH,
    pub masks: HashMap<String, XYWH>,
}

#[derive(Deserialize)]
pub struct FrameAtlas {
    size: [u32; 2],
    frames: HashMap<String, HashMap<String, Vec<Frame>>>,
}

impl FrameAtlas {
    pub fn new(file_path: &str) -> Self {
        let meta = fs::read_to_string(file_path).unwrap();

        serde_json::from_str(&meta).unwrap()
    }

    fn get_frame(&self, name: &str, tag: &str, idx: usize) -> &Frame {
        self.frames
            .get(name)
            .expect(&format!("FrameAtlas should contain {}", name))
            .get(tag)
            .expect(&format!(
                "FrameAtlas for {} should have tag {}",
                name, tag
            ))
            .get(idx)
            .expect(&format!(
                "FrameAtlas for {}.{} should have at least {} frames",
                name,
                tag,
                idx + 1
            ))
    }

    pub fn get_sprite_xywh(
        &self,
        name: &str,
        tag: &str,
        idx: usize,
    ) -> XYWH {
        self.get_frame(name, tag, idx).sprite
    }
}
