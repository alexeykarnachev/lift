use crate::vec::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

const MAX_N_ANIMATIONS: usize = 16;

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
struct FrameAtlasJson {
    size: [u32; 2],
    #[serde(rename = "frames")]
    frames: Vec<Frame>,
    index: HashMap<String, (usize, usize)>
}

#[derive(Default, Copy, Clone)]
pub struct AnimationParams {
    name: &'static str,
    id: usize,
    frame_duration: f32,
    is_repeat: bool,
}

impl AnimationParams {
    pub fn new(name: &'static str, id: usize, frame_duration: f32, is_repeat: bool) -> Self {
        Self { name, id, frame_duration, is_repeat }
    }
}

pub struct FrameAtlas {
    frames: &'static [Frame],
    name_to_slice: HashMap<String, &'static[Frame]>,
}

impl FrameAtlas {
    pub fn new(file_path: &str) -> Self {
        let meta = fs::read_to_string(file_path).unwrap();
        let atlas: FrameAtlasJson = serde_json::from_str(&meta).unwrap();
        let frames = atlas.frames.leak();

        let mut name_to_slice = HashMap::new();
        for (name, (start, n_frames)) in atlas.index {
            let slice = &frames[start..start + n_frames];
            name_to_slice.insert(name, slice);
        }

        Self {
            frames,
            name_to_slice,
        }
    }

    pub fn get_frame_animator(&self, animation_params: &[AnimationParams]) -> FrameAnimator {
        if animation_params.len() > MAX_N_ANIMATIONS {
            panic!("Can't create the Animator with more than {} animations", MAX_N_ANIMATIONS);
        }

        let mut prev_id = 0;
        let mut id_offset = 0;
        let mut frame_slices = [self.frames; MAX_N_ANIMATIONS];
        let mut animation_params_arr = [AnimationParams::default(); MAX_N_ANIMATIONS];
        for (i, params) in animation_params.iter().enumerate() {
            let id = params.id;
            if i == 0 {
                prev_id = id;
                id_offset = id;
            } else if id - prev_id != 1 {
                panic!("Animations must have a consecutive ids");
            }
            
            let slice = self.name_to_slice.get(params.name).expect(&format!("FrameAtlas should contain animation `{}`", params.name));
            frame_slices[i] = slice;
            animation_params_arr[i] = params.clone();
        }

        FrameAnimator {
            frame_slices,
            animation_params: animation_params_arr,
            n_animations: animation_params.len(),
            id_offset,
        }
    }
}

pub struct FrameAnimator {
    frame_slices: [&'static[Frame]; MAX_N_ANIMATIONS],
    animation_params: [AnimationParams; MAX_N_ANIMATIONS],
    n_animations: usize,
    id_offset: usize,
}

/*
pub struct FrameAnimator {
    n_frames: f32,
    frame_duration: f32,
    is_repeat: bool,
    progress: f32,
}

impl FrameAnimator {
    pub fn new(n_frames: usize, frame_duration: f32, is_repeat: bool) -> Self {
        Self {
            n_frames: n_frames as f32,
            frame_duration,
            is_repeat,
            progress: 0.0,
        }
    }

    pub fn is_finished(&self) -> bool {
        !self.is_repeat && self.progress == 1.0
    }

    pub fn update(&mut self, dt: f32) -> usize {
        self.progress += dt / (self.n_frames * self.frame_duration);
        if self.is_repeat {
            self.progress -= self.progress.floor();
        } else {
            self.progress = self.progress.min(1.0);
        };

        let idx = (self.progress * (self.n_frames - 1.0)).round() as usize;

        idx
    }
}
*/
