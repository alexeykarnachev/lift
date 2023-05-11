use crate::entity::*;
use crate::graphics::*;
use crate::input::*;
use crate::vec::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
// use serde_json::from_value;

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
    pub id_to_text: HashMap<String, Text>,
}

impl UI {
    pub fn from_file(
        file_path: &'static str,
        glyph_atlas: &GlyphAtlas,
    ) -> Self {
        let data = fs::read_to_string(file_path).unwrap();
        let elements: Vec<Element> = serde_json::from_str(&data).unwrap();

        let mut id_to_text = HashMap::new();
        for element in elements {
            let position_config = element.position;
            let origin =
                Origin::from_str(&position_config.origin, Vec2::zeros());

            let position = Vec2::new(position_config.x, position_config.y);

            let text = if let Some(text_config) = element.text {
                Text::new(
                    position,
                    &glyph_atlas,
                    Space::Screen,
                    origin,
                    text_config.string,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                    text_config.scale,
                )
            } else {
                panic!(
                    "UI element {:?} doesn't have a text field",
                    element.id
                );
            };

            id_to_text.insert(element.id, text);
        }

        Self {
            file_path,
            id_to_text,
        }
    }

    pub fn process_input(&mut self, input: &Input) -> Option<UIEvent> {
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
        for (id, text) in self.id_to_text.iter_mut() {
            let rect = text.get_bound_rect();
            if rect.collide_with_point(cursor_pos) {
                text.set_color(Color::yellow(1.0));

                let id = id.clone();
                if input.lmb_press_pos.is_some() {
                    event = Some(UIEvent::LMBPress(id));
                } else if input.rmb_press_pos.is_some() {
                    event = Some(UIEvent::RMBPress(id));
                } else {
                    event = Some(UIEvent::Hover(id));
                }
            } else {
                text.set_color(Color::red(1.0));
            }
        }

        event
    }
}
