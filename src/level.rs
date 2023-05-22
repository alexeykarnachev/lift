use crate::entity::*;
use crate::graphics::*;
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
    pub player: Entity,
    pub enemies: Vec<Entity>,
    pub colliders: Vec<Collider>,
    pub draw_primitives: Vec<DrawPrimitive>,
    pub lights: Vec<Entity>,
}

#[derive(Copy, Clone, Debug)]
pub enum Collider {
    Rigid(Rect),
    Stair(Rect),
}

impl Level {
    pub fn new(file_path: &str, sprite_atlas: &SpriteAtlas) -> Self {
        use EffectType::*;

        let meta = fs::read_to_string(file_path).unwrap();
        let tiled_json: TiledJson = serde_json::from_str(&meta).unwrap();
        let tilewidth = tiled_json.tilewidth;
        let tileheight = tiled_json.tileheight;
        let global_width = tiled_json.width as f32 * tilewidth;
        let global_height = tiled_json.height as f32 * tileheight;

        let mut player = None;
        let mut colliders = Vec::new();
        let mut enemies = Vec::new();
        let mut lights = Vec::new();
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
                                    SpaceType::WorldSpace,
                                    0.0,
                                    ApplyLightEffect as u32,
                                    position,
                                    sprite,
                                    None,
                                    false,
                                    TextureType::SpriteTexture,
                                );
                                draw_primitives.push(primitive);
                            }
                        }
                    }
                }
                "objects" => {
                    let objects = layer.objects.as_ref().unwrap();
                    for object in objects {
                        let position =
                            Vec2::new(object.x, global_height - object.y);
                        let rect = Rect::from_top_left(
                            position,
                            Vec2::new(object.width, object.height),
                        );
                        match object.name.as_str() {
                            "collider" => {
                                colliders.push(Collider::Rigid(rect));
                            }
                            "stair" => {
                                colliders.push(Collider::Stair(rect));
                            }
                            "stone_wall" => {
                                let stone_wall = create_stone_wall(rect);
                                draw_primitives.push(stone_wall);
                            }
                            "player" => {
                                player = Some(create_player(
                                    position,
                                    sprite_atlas,
                                ));
                            }
                            "rat" => {
                                let rat =
                                    create_rat(position, sprite_atlas);
                                spawn_enemy(rat, &mut enemies);
                            }
                            "bat" => {
                                let bat =
                                    create_bat(position, sprite_atlas);
                                spawn_enemy(bat, &mut enemies);
                            }
                            "rat_nest" => {
                                let rat_nest = create_rat_nest(
                                    position,
                                    sprite_atlas,
                                );
                                spawn_enemy(rat_nest, &mut enemies);
                            }
                            "torch" => {
                                let torch =
                                    create_torch(position, sprite_atlas);
                                lights.push(torch);
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
            player: player.unwrap(),
            enemies,
            colliders,
            draw_primitives,
            lights,
        }
    }

    pub fn spawn_enemy(&mut self, enemy: Entity) {
        spawn_enemy(enemy, &mut self.enemies);
    }
}

fn spawn_enemy(mut enemy: Entity, array: &mut Vec<Entity>) {
    enemy.id = array.len() as i32;
    array.push(enemy);
}
