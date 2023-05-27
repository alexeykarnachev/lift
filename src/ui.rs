use crate::entity::*;
use crate::graphics::*;
use crate::input::*;
use crate::vec::*;
use serde::Deserialize;
use std::fs;

pub enum UIEvent {
    Hover(String),
    LMBPress(String),
    RMBPress(String),
    Empty(String),
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

    #[serde(skip)]
    color: Color,
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
            if position.x <= 1.0 {
                position.x =
                    (position.x * window_size.x + window_size.x) * 0.5;
            }
            if position.y <= 1.0 {
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

                    if let Some(filling) = element.filling {
                        width *= filling;
                    }
                    if width <= 1.0 {
                        width *= window_size.x;
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
                        0,
                        element.color,
                    );
                    collider = rect;
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
                        element.color = Color::new(0.9, 0.9, 0.5, 1.0);
                    }
                } else {
                    element.color = Color::default();
                }
            }
        }

        events
    }
}

const WINDOW_X: f32 = 10.0;
const WINDOW_Y: f32 = 10.0;
const WINDOW_WIDTH: f32 = 500.0;
const WINDOW_HEIGHT: f32 = 500.0;
const WINDOW_BORDER_SIZE: f32 = 10.0;

const SKILL_WIDTH: f32 = 50.0;
const SKILL_HEIGHT: f32 = 50.0;
const SKILL_PAD_SIZE: f32 = 25.0;

pub fn create_skill_tree_ui() -> UI {
    let mut cursor = Vec2::new(WINDOW_X, WINDOW_Y);

    let window = Element {
        id: "window".to_string(),
        type_: "rect".to_string(),
        is_interactive: false,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        width: Some(WINDOW_WIDTH),
        height: Some(WINDOW_HEIGHT),
        color: Color::gray(0.1, 1.0),
        ..Default::default()
    };
    cursor += Vec2::new(WINDOW_BORDER_SIZE, WINDOW_BORDER_SIZE);

    // Attack line
    let mut attack_line_cursor = cursor;
    let attack_line = vec![
        create_skill_rect("attack_0", &mut attack_line_cursor),
        create_skill_rect("attack_1", &mut attack_line_cursor),
        create_skill_rect("attack_2", &mut attack_line_cursor),
    ];
    cursor.y += SKILL_HEIGHT + SKILL_PAD_SIZE;

    // Durability line
    let mut durability_line_cursor = cursor;
    let durability_line = vec![
        create_skill_rect("durability_0", &mut durability_line_cursor),
        create_skill_rect("durability_1", &mut durability_line_cursor),
        create_skill_rect("durability_2", &mut durability_line_cursor),
    ];
    cursor.y += SKILL_HEIGHT + SKILL_PAD_SIZE;

    // Agility line
    let mut agility_line_cursor = cursor;
    let agility_line = vec![
        create_skill_rect("agility_0", &mut agility_line_cursor),
        create_skill_rect("agility_1", &mut agility_line_cursor),
        create_skill_rect("agility_2", &mut agility_line_cursor),
    ];
    cursor.y += SKILL_HEIGHT + SKILL_PAD_SIZE;

    // Footer
    let skill_points_text = Element {
        id: "skill_points_text".to_string(),
        type_: "text".to_string(),
        is_interactive: false,
        position: Position {
            origin: "BotLeft".to_string(),
            x: 20.0,
            y: 500.0,
        },
        string: Some("Points: 228".to_string()),
        font_size: Some(28),

        color: Color::gray(0.5, 1.0),
        ..Default::default()
    };

    // let elements = vec![window, attack_0, attack_1, skill_points_text];
    let mut elements = vec![window];
    elements.extend_from_slice(&attack_line);
    elements.extend_from_slice(&durability_line);
    elements.extend_from_slice(&agility_line);
    elements.push(skill_points_text);

    UI::new(elements)
}

fn create_skill_rect(name: &str, cursor: &mut Vec2<f32>) -> Element {
    let element = Element {
        id: name.to_string(),
        type_: "rect".to_string(),
        is_interactive: true,
        position: Position {
            origin: "TopLeft".to_string(),
            x: cursor.x,
            y: cursor.y,
        },
        width: Some(SKILL_WIDTH),
        height: Some(SKILL_HEIGHT),
        color: Color::gray(0.3, 1.0),
        ..Default::default()
    };

    cursor.x += SKILL_WIDTH + SKILL_PAD_SIZE;

    element
}
