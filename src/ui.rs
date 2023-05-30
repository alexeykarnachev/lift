#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::entity::*;
use crate::graphics::*;
use crate::input::*;
use crate::player_stats::SkillsChain;
use crate::player_stats::Stats;
use crate::vec::*;
use serde::Deserialize;
use std::fs;

mod play_ui {
    pub mod window {
        // Top left
        pub const X: f32 = 10.0;
        pub const Y: f32 = -90.0;
        pub const WIDTH: f32 = 300.0;
        pub const HEIGHT: f32 = 80.0;
        pub const BORDER_SIZE: f32 = 10.0;
    }
}

mod skill_tree_ui {
    pub mod window {
        // Top left
        pub const X: f32 = 10.0;
        pub const Y: f32 = 10.0;
        pub const WIDTH: f32 = 470.0;
        pub const HEIGHT: f32 = 500.0;
        pub const BORDER_SIZE: f32 = 20.0;
    }

    pub mod skill {
        pub const HPAD_SIZE: f32 = 90.0;
        pub const VPAD_SIZE: f32 = 100.0;
    }
}

#[derive(Debug)]
pub enum UIEvent {
    Hover(String),
    LMBPress(String),
    RMBPress(String),
    NotInteracted(String),
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Position {
    origin: String,
    x: f32,
    y: f32,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Element {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    is_interactive: bool,
    position: Position,

    // Text
    string: Option<String>,
    font_size: Option<u32>,

    // Rect
    width: Option<f32>,
    height: Option<f32>,
    aspect: Option<f32>,
    filling: Option<f32>,

    // Sprite
    sprite_name: Option<String>,
    sprite_idx: Option<usize>,
    sprite_scale: Option<f32>,

    #[serde(skip)]
    color: Color,
    #[serde(skip)]
    pub effect: u32,
}

pub struct UI {
    pub texts: Vec<Text>,
    pub rects: Vec<DrawPrimitive>,
    pub elements: Vec<Element>,
}

impl UI {
    pub fn new(elements: Vec<Element>) -> Self {
        let texts = Vec::<Text>::with_capacity(elements.len());
        let rects = Vec::<DrawPrimitive>::with_capacity(elements.len());

        Self {
            elements,
            texts,
            rects,
        }
    }

    pub fn from_file(file_path: &'static str) -> Self {
        let data = fs::read_to_string(file_path).unwrap();
        let elements: Vec<Element> = serde_json::from_str(&data).unwrap();

        Self::new(elements)
    }

    pub fn set_element_string(&mut self, element_id: &str, string: &str) {
        for element in self.elements.iter_mut() {
            if element.id == element_id {
                element.string = Some(string.to_string());
                return;
            }
        }

        panic!("No such element: {:?}", element_id);
    }

    pub fn set_element_color(&mut self, element_id: &str, color: Color) {
        for element in self.elements.iter_mut() {
            if element.id == element_id {
                element.color = color;
                return;
            }
        }

        panic!("No such element: {:?}", element_id);
    }

    pub fn set_element_effect(&mut self, element_id: &str, effect: u32) {
        for element in self.elements.iter_mut() {
            if element.id == element_id {
                element.effect = effect;
                return;
            }
        }

        panic!("No such element: {:?}", element_id);
    }

    pub fn set_element_filling(&mut self, element_id: &str, filling: f32) {
        for element in self.elements.iter_mut() {
            if element.id == element_id {
                element.filling = Some(filling);
                return;
            }
        }

        panic!("No such element: {:?}", element_id);
    }

    pub fn update(
        &mut self,
        input: &Input,
        glyph_atlas: &GlyphAtlas,
        sprite_atlas: &SpriteAtlas,
    ) -> Vec<UIEvent> {
        let cursor_pos = Vec2::new(
            input.cursor_pos.x as f32,
            (input.window_size.y - input.cursor_pos.y) as f32,
        );
        let window_size = Vec2::new(
            input.window_size.x as f32,
            input.window_size.y as f32,
        );
        let window_aspect = window_size.x / window_size.y;

        self.texts.clear();
        self.rects.clear();
        let mut events = Vec::with_capacity(self.elements.len());
        for element in self.elements.iter_mut() {
            let origin = Origin::from_str(&element.position.origin);
            let mut position =
                Vec2::new(element.position.x, element.position.y);

            if position.x < -1.0 {
                position.x = window_size.x - position.x;
            } else if position.x <= 1.0 {
                position.x =
                    (position.x * window_size.x + window_size.x) * 0.5;
            }

            if position.y < -1.0 {
                position.y *= -1.0;
            } else if position.y <= 1.0 {
                position.y =
                    (position.y * window_size.y + window_size.y) * 0.5;
            } else {
                position.y = window_size.y - position.y;
            }

            let collider;
            match element.type_.as_str() {
                "text" => {
                    let string = element.string.clone().unwrap();
                    let font_size = element.font_size.unwrap();
                    let text = Text::new(
                        position,
                        &glyph_atlas,
                        SpaceType::ScreenSpace,
                        origin,
                        string.clone(),
                        font_size,
                        element.color,
                    );
                    collider = text.get_bound_rect();
                    self.texts.push(text);
                }
                "rect" => {
                    let mut width;
                    let mut height;
                    if element.aspect.is_none() {
                        width = element.width.unwrap();
                        height = element.height.unwrap();
                    } else if element.height.is_none() {
                        width = element.width.unwrap();
                        height = window_aspect * width
                            / element.aspect.unwrap();
                    } else if element.width.is_none() {
                        height = element.height.unwrap();
                        width = height * element.aspect.unwrap()
                            / window_aspect;
                    } else {
                        panic!("Element's width, height and aspect can't be all set at the same time. One a pair of these three parameters could be set");
                    }

                    if width <= 1.0 {
                        width *= window_size.x;
                    }
                    if let Some(filling) = element.filling {
                        width *= filling;
                    }
                    if height <= 1.0 {
                        height *= window_size.y;
                    }

                    let size = Vec2::new(width, height);
                    let rect = Rect::from_origin(origin, position, size);
                    let primitive = DrawPrimitive::from_rect(
                        rect,
                        SpaceType::ScreenSpace,
                        0.0,
                        element.effect,
                        element.color,
                    );
                    collider = rect;
                    self.rects.push(primitive);
                }
                "sprite" => {
                    let name = element.sprite_name.as_ref().unwrap();
                    let idx = element.sprite_idx.unwrap();
                    let scale = element.sprite_scale.unwrap();
                    let mut sprite =
                        sprite_atlas.sprites.get(name).unwrap()[idx];
                    sprite.origin = origin;

                    let primitive = DrawPrimitive::from_sprite(
                        SpaceType::ScreenSpace,
                        0.0,
                        element.effect,
                        position,
                        sprite,
                        Some(element.color),
                        false,
                        TextureType::SpriteTexture,
                        scale,
                    );
                    collider = primitive.rect;
                    self.rects.push(primitive);
                }
                _ => {
                    panic!("Unknown UI element type: {:?}", element.type_)
                }
            }

            if element.is_interactive {
                let id = element.id.clone();

                if collider.collide_with_point(cursor_pos) {
                    if input.lmb_press_pos.is_some() {
                        events.push(UIEvent::LMBPress(id));
                    } else if input.rmb_press_pos.is_some() {
                        events.push(UIEvent::RMBPress(id));
                    } else {
                        events.push(UIEvent::Hover(id));
                    }
                } else {
                    events.push(UIEvent::NotInteracted(id));
                }
            }
        }

        events
    }
}

pub fn create_main_menu_ui() -> UI {
    let window = Element {
        id: "window".to_string(),
        type_: "rect".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: -1.0,
            y: 1.0,
        },
        width: Some(1.0),
        height: Some(1.0),
        color: Color::gray(0.0, 0.85),
        ..Default::default()
    };

    let mut cursor = Vec2::new(30.0, -30.0);
    let quit_text = Element {
        id: "quit_text".to_string(),
        type_: "text".to_string(),
        is_interactive: true,
        position: Position {
            origin: "BotLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        string: Some("Quit".to_string()),
        font_size: Some(37),
        color: Color::gray(0.5, 1.0),
        ..Default::default()
    };
    cursor.y -= 50.0;

    let options_text = Element {
        id: "options_text".to_string(),
        type_: "text".to_string(),
        is_interactive: true,
        position: Position {
            origin: "BotLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        string: Some("Options".to_string()),
        font_size: Some(37),
        color: Color::gray(0.5, 1.0),
        ..Default::default()
    };
    cursor.y -= 50.0;

    let new_game_text = Element {
        id: "new_game_text".to_string(),
        type_: "text".to_string(),
        is_interactive: true,
        position: Position {
            origin: "BotLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        string: Some("New Game".to_string()),
        font_size: Some(37),
        color: Color::gray(0.5, 1.0),
        ..Default::default()
    };

    let elements = vec![window, new_game_text, options_text, quit_text];

    UI::new(elements)
}

pub fn create_play_ui() -> UI {
    use play_ui::*;

    let mut cursor = Vec2::new(window::X, window::Y);

    let window = Element {
        id: "window".to_string(),
        type_: "rect".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        width: Some(window::WIDTH),
        height: Some(window::HEIGHT),
        color: Color::gray(0.1, 0.5),
        ..Default::default()
    };
    cursor += Vec2::new(window::BORDER_SIZE, window::BORDER_SIZE);

    let level_number_rect_size =
        window::HEIGHT - 2.0 * window::BORDER_SIZE;
    let level_number_rect = Element {
        id: "level_number_rect".to_string(),
        type_: "rect".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        width: Some(level_number_rect_size),
        height: Some(level_number_rect_size),
        color: Color::expbar(1.0),
        ..Default::default()
    };
    cursor.x += level_number_rect_size + window::BORDER_SIZE;

    let mut level_number_center = level_number_rect.position.clone();
    level_number_center.x += 0.5 * level_number_rect_size;
    level_number_center.y += 0.5 * level_number_rect_size;
    let level_number = Element {
        id: "level_number".to_string(),
        type_: "text".to_string(),
        is_interactive: false,
        position: Position {
            origin: "Center".to_string(),
            x: level_number_center.x,
            y: level_number_center.y,
        },
        string: Some("0".to_string()),
        font_size: Some(28),
        color: Color::gray(0.1, 1.0),
        ..Default::default()
    };

    let healthbar_width =
        window::WIDTH - level_number_rect_size - 3.0 * window::BORDER_SIZE;
    let healthbar_height = 15.0;
    let healthbar = Element {
        id: "healthbar".to_string(),
        type_: "rect".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        width: Some(healthbar_width),
        height: Some(healthbar_height),
        color: Color::healthbar(1.0),
        ..Default::default()
    };
    cursor.y += healthbar_height + window::BORDER_SIZE;

    let staminabar = Element {
        id: "staminabar".to_string(),
        type_: "rect".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        width: Some(healthbar_width),
        height: Some(healthbar_height),
        color: Color::staminabar(1.0),
        ..Default::default()
    };
    cursor.y += healthbar_height + window::BORDER_SIZE;
    cursor.x -= window::BORDER_SIZE;

    let expbar_width = healthbar_width + window::BORDER_SIZE;
    let expbar_height = level_number_rect_size
        - 2.0 * window::BORDER_SIZE
        - 2.0 * healthbar_height;
    let expbar = Element {
        id: "expbar".to_string(),
        type_: "rect".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        width: Some(expbar_width),
        height: Some(expbar_height),
        color: Color::expbar(1.0),
        ..Default::default()
    };

    let elements = vec![
        window,
        level_number_rect,
        level_number,
        healthbar,
        staminabar,
        expbar,
    ];

    UI::new(elements)
}

pub fn create_skill_tree_ui(
    sprite_atlas: &SpriteAtlas,
    stats: &Stats,
) -> UI {
    use skill_tree_ui::*;

    let mut cursor = Vec2::new(window::X, window::Y);

    let window = Element {
        id: "window".to_string(),
        type_: "rect".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        width: Some(window::WIDTH),
        height: Some(window::HEIGHT),
        color: Color::gray(0.1, 1.0),
        ..Default::default()
    };
    cursor += Vec2::new(window::BORDER_SIZE, window::BORDER_SIZE);

    // Header
    let skill_points_text = Element {
        id: "skill_points_text".to_string(),
        type_: "text".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        string: Some("Points: 0".to_string()),
        font_size: Some(28),

        color: Color::gray(0.5, 1.0),
        ..Default::default()
    };
    cursor.y += 0.5 * skill::VPAD_SIZE;

    // Attack line
    let mut attack_line_cursor = cursor;
    let attack_line = create_skills_chain(
        &mut attack_line_cursor,
        &stats.attack_skills,
        "attack_skills",
    );
    cursor.y += skill::VPAD_SIZE;

    // Durability line
    let mut durability_line_cursor = cursor;
    let durability_line = create_skills_chain(
        &mut durability_line_cursor,
        &stats.durability_skills,
        "durability_skills",
    );
    cursor.y += skill::VPAD_SIZE;

    // Agility line
    let mut agility_line_cursor = cursor;
    let agility_line = create_skills_chain(
        &mut agility_line_cursor,
        &stats.agility_skills,
        "agility_skills",
    );
    cursor.y += skill::VPAD_SIZE;

    // Light line
    let mut light_line_cursor = cursor;
    let light_line = create_skills_chain(
        &mut light_line_cursor,
        &stats.light_skills,
        "light_skills",
    );
    cursor.y += skill::VPAD_SIZE;

    // Footer
    let mut cursor = Vec2::new(window::X, window::Y);
    cursor += Vec2::new(
        window::BORDER_SIZE,
        window::HEIGHT - window::BORDER_SIZE,
    );
    let skill_description_text = Element {
        id: "skill_description_text".to_string(),
        type_: "text".to_string(),
        is_interactive: false,
        position: Position {
            origin: "BotLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        string: Some(" ".to_string()),
        font_size: Some(16),

        color: Color::gray(0.5, 1.0),
        ..Default::default()
    };

    let mut elements = vec![];
    elements.push(window);
    elements.push(skill_points_text);
    elements.extend_from_slice(&attack_line);
    elements.extend_from_slice(&durability_line);
    elements.extend_from_slice(&agility_line);
    elements.extend_from_slice(&light_line);
    elements.push(skill_description_text);

    UI::new(elements)
}

fn create_skills_chain(
    cursor: &mut Vec2<f32>,
    skills_chain: &SkillsChain,
    sprite_name: &str,
) -> Vec<Element> {
    use skill_tree_ui::*;
    use EffectType::*;

    let mut elements = vec![];
    let n_learned = skills_chain.n_learned;

    for (idx, skill) in skills_chain.skills.iter().enumerate() {
        let mut sprite_id = sprite_name.to_string();
        sprite_id.push_str("_");
        sprite_id.push_str(&idx.to_string());

        if idx > 0 {
            let mut arrow_id = "arrow_".to_string();
            arrow_id.push_str(&sprite_id.to_string());
            let element = Element {
                id: arrow_id,
                type_: "sprite".to_string(),
                is_interactive: false,
                position: Position {
                    origin: "TopLeft".to_string(),
                    x: cursor.x,
                    y: cursor.y,
                },
                sprite_name: Some("skills_arrow".to_string()),
                sprite_idx: Some(0),
                sprite_scale: Some(4.0),
                color: Color::only_alpha(0.1),
                ..Default::default()
            };
            elements.push(element);
            cursor.x += skill::HPAD_SIZE;
        }

        let element = Element {
            id: sprite_id,
            type_: "sprite".to_string(),
            is_interactive: true,
            position: Position {
                origin: "TopLeft".to_string(),
                x: cursor.x,
                y: cursor.y,
            },
            sprite_name: Some(sprite_name.to_string()),
            sprite_idx: Some(idx),
            sprite_scale: Some(4.0),
            color: Color::only_alpha(0.1),
            ..Default::default()
        };
        elements.push(element);
        cursor.x += skill::HPAD_SIZE;
    }

    elements
}
