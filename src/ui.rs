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
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextConfig {
    string: String,
    scale: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PositionConfig {
    origin: String,
    x: f32,
    y: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Element {
    id: String,
    position: PositionConfig,
    text: Option<TextConfig>,
}

pub struct UI {
    pub file_path: &'static str,
    pub texts: Vec<Text>,
    elements: Vec<Element>,
}

impl UI {
    pub fn new(file_path: &'static str) -> Self {
        let data = fs::read_to_string(file_path).unwrap();
        let elements: Vec<Element> = serde_json::from_str(&data).unwrap();
        let texts = Vec::<Text>::with_capacity(elements.len());

        Self {
            file_path,
            elements,
            texts,
        }
    }

    pub fn set_element_text(&mut self, element_id: &str, string: &str) {
        for element in self.elements.iter_mut() {
            if element.id == element_id {
                element.text.as_mut().unwrap().string = string.to_string();
                return;
            }
        }

        panic!("No such element: {:?}", element_id);
    }

    pub fn update(
        &mut self,
        input: &Input,
        glyph_atlas: &GlyphAtlas,
    ) -> Option<UIEvent> {
        let cursor_pos = Vec2::new(
            input.cursor_pos.x as f32,
            (input.window_size.y - input.cursor_pos.y) as f32,
        );
        let window_size = Vec2::new(
            input.window_size.x as f32,
            input.window_size.y as f32,
        );
        let cursor_pos = cursor_pos - window_size.scale(0.5);

        let mut event = None;
        self.texts.clear();
        for element in &self.elements {
            let origin =
                Origin::from_str(&element.position.origin, Vec2::zeros());

            let mut position =
                Vec2::new(element.position.x, element.position.y);
            position = (position * window_size).scale(0.5);

            let mut text = if let Some(text_config) = &element.text {
                Text::new(
                    position,
                    &glyph_atlas,
                    Space::Screen,
                    origin,
                    text_config.string.clone(),
                    Color::new(1.0, 0.0, 0.0, 1.0),
                    text_config.scale,
                )
            } else {
                panic!(
                    "UI element {:?} doesn't have a text field",
                    element.id
                );
            };

            let rect = text.get_bound_rect();
            if rect.collide_with_point(cursor_pos) {
                text.set_color(Color::yellow(1.0));

                let id = element.id.clone();
                if input.lmb_press_pos.is_some() {
                    event = Some(UIEvent::LMBPress(id));
                } else if input.rmb_press_pos.is_some() {
                    event = Some(UIEvent::RMBPress(id));
                } else {
                    event = Some(UIEvent::Hover(id));
                }
            }

            self.texts.push(text);
        }

        event
    }
}
