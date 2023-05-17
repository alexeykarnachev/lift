use crate::entity::*;
use crate::graphics::{DrawPrimitive, Space, SpriteAtlas, Texture};
use crate::prefabs::*;
use crate::vec::{Origin, Rect, Vec2};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct TiledJson {
    width: usize,
    height: usize,
    tilewidth: f32,
    tileheight: f32,
    layers: Vec<TiledLayerJson>,
}

#[derive(Deserialize)]
struct TiledLayerJson {
    data: Option<Vec<usize>>,
    objects: Option<Vec<TiledObjectJson>>,
    height: Option<usize>,
    width: Option<usize>,
    opacity: f32,
    name: String,

    #[serde(rename = "type")]
    type_: String,
    x: usize,
    y: usize,
}

#[derive(Deserialize)]
struct TiledObjectJson {
    x: f32,
    y: f32,
    point: Option<bool>,
    width: f32,
    height: f32,
    name: String,
}

pub struct Level {
    pub room: Rect,
    pub player: Entity,
    pub enemies: Vec<Entity>,
    pub draw_primitives: Vec<DrawPrimitive>,
}

impl Level {
    pub fn new(file_path: &str, sprite_atlas: &SpriteAtlas) -> Self {
        let meta = fs::read_to_string(file_path).unwrap();
        let tiled_json: TiledJson = serde_json::from_str(&meta).unwrap();
        let tilewidth = tiled_json.tilewidth;
        let tileheight = tiled_json.tileheight;
        let global_width = tiled_json.width as f32 * tilewidth;
        let global_height = tiled_json.height as f32 * tileheight;

        let mut room = None;
        let mut player = None;
        let mut enemies = Vec::new();
        let mut draw_primitives = Vec::new();

        let layers = tiled_json.layers;
        for layer in layers.iter() {
            match layer.name.as_str() {
                "tiles" => {
                    let data = layer.data.as_ref().unwrap();
                    let sprites = &sprite_atlas.sprites["tilemap"];
                    let n_cols = layer.width.unwrap();
                    let n_rows = layer.height.unwrap();
                    let width = tilewidth * n_cols as f32;
                    let height = tileheight * n_rows as f32;
                    for i in 0..n_rows {
                        for j in 0..n_cols {
                            let idx = data[i * n_cols + j];
                            if idx != 0 {
                                let x = j as f32 * tilewidth;
                                let y = height - i as f32 * tileheight;
                                let position = Vec2::new(x, y);
                                let mut sprite = sprites[idx - 1];
                                sprite.origin = Origin::TopLeft;

                                let primitive = DrawPrimitive::from_sprite(
                                    Space::World,
                                    position,
                                    sprite,
                                    None,
                                    false,
                                    Texture::Sprite,
                                );
                                draw_primitives.push(primitive);
                            }
                        }
                    }
                }
                "objects" => {
                    let objects = layer.objects.as_ref().unwrap();
                    for object in objects {
                        match object.name.as_str() {
                            "room" => {
                                let position =
                                    Vec2::new(object.x, object.y);
                                let size =
                                    Vec2::new(object.width, object.height);
                                room = Some(Rect::from_bot_left(
                                    position, size,
                                ));
                            }
                            "player" => {
                                if let Some(true) = object.point {
                                    let position = Vec2::new(
                                        object.x,
                                        global_height - object.y,
                                    );
                                    player = Some(create_player(
                                        position,
                                        sprite_atlas,
                                    ));
                                } else {
                                    panic!("Player object must have point type");
                                }
                            }
                            "rat" => {
                                if let Some(true) = object.point {
                                    let position = Vec2::new(
                                        object.x,
                                        global_height - object.y,
                                    );
                                    let rat =
                                        create_rat(position, sprite_atlas);
                                    enemies.push(rat);
                                } else {
                                    panic!(
                                        "Rat object must have point type"
                                    );
                                }
                            }
                            "bat" => {
                                if let Some(true) = object.point {
                                    let position = Vec2::new(
                                        object.x,
                                        global_height - object.y,
                                    );
                                    let bat =
                                        create_bat(position, sprite_atlas);
                                    enemies.push(bat);
                                } else {
                                    panic!(
                                        "Bat object must have point type"
                                    );
                                }
                            }
                            _ => {
                                panic!(
                                    "Unhandled Tiled layer object: {:?}",
                                    object.name
                                );
                            }
                        }
                    }
                }
                _ => {
                    panic!("Unhandled Tiled layer: {:?}", layer.name);
                }
            };
        }

        Self {
            room: room.unwrap(),
            player: player.unwrap(),
            enemies,
            draw_primitives,
        }
    }
}
