use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize)]
struct XYWH {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Deserialize)]
struct FrameJson {
    pub sprite: XYWH,
    pub rigid_collider: Option<XYWH>,
    pub attack_collider: Option<XYWH>,
}

#[derive(Deserialize)]
struct AtlasJson {
    pub size: [u32; 2],
    pub frames: HashMap<String, Vec<FrameJson>>,
}

pub struct SpriteAtlas {
    pub size: [u32; 2],

}
