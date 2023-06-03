use image::imageops::flip_vertical_in_place;
use image::io::Reader as ImageReader;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
pub struct XYWH {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Deserialize)]
pub struct Frame {
    pub sprite: XYWH,
    pub rigid_collider: Option<XYWH>,
    pub attack_collider: Option<XYWH>,
}

#[derive(Deserialize)]
pub struct FrameAtlas {
    pub size: [u32; 2],
    pub frames: HashMap<String, Vec<Frame>>,

    #[serde(skip)]
    pub image: Vec<u8>,
}

impl FrameAtlas {
    pub fn new(meta_file_path: &str, image_file_path: &str) -> Self {
        let meta = fs::read_to_string(meta_file_path).unwrap();
        let mut atlas: Self = serde_json::from_str(&meta).unwrap();

        let mut image = ImageReader::open(image_file_path)
            .unwrap()
            .decode()
            .unwrap();
        flip_vertical_in_place(&mut image);

        atlas.image = image.as_bytes().to_vec();

        atlas
    }
}
