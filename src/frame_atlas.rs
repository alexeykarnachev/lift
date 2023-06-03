use crate::vec::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Copy, Clone)]
struct XYWH {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl XYWH {
    pub fn to_array(&self) -> [f32; 4] {
        [self.x as f32, self.y as f32, self.w as f32, self.h as f32]
    }
}

#[derive(Deserialize, Copy, Clone)]
struct Frame {
    pub sprite: XYWH,
    pub rigid_collider: Option<XYWH>,
    pub attack_collider: Option<XYWH>,
}

#[derive(Deserialize)]
pub struct FrameAtlas {
    size: [u32; 2],
    frames: HashMap<String, Vec<Frame>>,
}

impl FrameAtlas {
    pub fn new(file_path: &str) -> Self {
        let meta = fs::read_to_string(file_path).unwrap();

        serde_json::from_str(&meta).unwrap()
    }

    fn get_frame(&self, name: &str, idx: usize) -> Frame {
        *self
            .frames
            .get(name)
            .expect(&format!("FrameAtlas should contain {:?}", name))
            .get(idx)
            .expect(&format!(
                "FrameAtlas for {:?} should have enought frames",
                name
            ))
    }

    pub fn get_sprite_xywh(&self, name: &str, idx: usize) -> [f32; 4] {
        self.get_frame(name, idx).sprite.to_array()
    }

    pub fn get_rigid_collider(
        &self,
        name: &str,
        idx: usize,
    ) -> Option<Rect> {
        if let Some(xywh) = self.get_frame(name, idx).rigid_collider {
            return Some(Rect::from_xywh(&xywh.to_array()));
        }

        None
    }

    pub fn get_attack_collider(
        &self,
        name: &str,
        idx: usize,
    ) -> Option<Rect> {
        if let Some(xywh) = self.get_frame(name, idx).attack_collider {
            return Some(Rect::from_xywh(&xywh.to_array()));
        }

        None
    }
}
