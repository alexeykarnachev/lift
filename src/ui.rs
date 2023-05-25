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

#[derive(Deserialize, Debug, Clone)]
pub struct Position {
    origin: String,
    x: f32,
    y: f32,
}

#[derive(Deserialize, Debug, Clone)]
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
    filling: Option<f32>,

    #[serde(skip)]
    color: Color,
}

pub struct UI {
    pub file_path: &'static str,
    pub texts: Vec<Text>,
    pub rects: Vec<DrawPrimitive>,
    pub elements: Vec<Element>,
}

impl UI {
    pub fn new(file_path: &'static str) -> Self {
        let data = fs::read_to_string(file_path).unwrap();
        let elements: Vec<Element> = serde_json::from_str(&data).unwrap();
        let texts = Vec::<Text>::with_capacity(elements.len());
        let rects = Vec::<DrawPrimitive>::with_capacity(elements.len());

        Self {
            file_path,
            elements,
            texts,
            rects,
        }
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

        self.texts.clear();
        self.rects.clear();
        let mut events = Vec::with_capacity(self.elements.len());
        for element in self.elements.iter_mut() {
            let origin = Origin::from_str(&element.position.origin);
            let mut position =
                Vec2::new(element.position.x, element.position.y);
            position = (position * window_size + window_size).scale(0.5);

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
                    let mut width = element.width.unwrap();
                    let height = element.height.unwrap();
                    if let Some(filling) = element.filling {
                        width *= filling;
                    }

                    let size = Vec2::new(width, height) * window_size;
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
