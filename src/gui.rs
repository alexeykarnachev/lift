use crate::entity::Text;
use crate::graphics::*;
use crate::input::*;
use crate::vec::*;

mod defaults {
    pub const VERTICAL_SPACING: f32 = 10.0;
    pub const HORIZONTAL_SPACING: f32 = 10.0;
    pub const FONT_SIZE: u32 = 28;
    pub const TEXT_RGBA: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
    pub const BAR_WIDTH: f32 = 310.0;
    pub const BAR_HEIGHT: f32 = 25.0;
}

#[derive(Default)]
pub enum DrawDirection {
    #[default]
    Down,
    Up,
    Right,
    Left,
}

#[derive(Default)]
pub struct GUI {
    ui_cursor: Vec2<f32>,
    mouse_cursor: Vec2<f32>,
    window_size: Vec2<f32>,

    draw_direction: DrawDirection,
    vertical_spacing: f32,
    horizontal_spacing: f32,
    font_size: u32,
    text_color: Color,
    bar_size: Vec2<f32>,
    effect: u32,

    primitives: Vec<DrawPrimitive>,
    texts: Vec<Text>,
}

impl GUI {
    pub fn new() -> Self {
        let primitives = Vec::with_capacity(1024);

        Self {
            primitives,
            ..Default::default()
        }
    }

    pub fn begin(&mut self, input: &mut Input) {
        self.primitives.clear();
        self.texts.clear();

        use defaults::*;
        self.draw_direction = DrawDirection::Down;
        self.vertical_spacing = VERTICAL_SPACING;
        self.horizontal_spacing = HORIZONTAL_SPACING;
        self.font_size = FONT_SIZE;
        self.text_color = Color::from_slice(&TEXT_RGBA);
        self.bar_size = Vec2::new(BAR_WIDTH, BAR_HEIGHT);
        self.effect = 0;

        self.ui_cursor.x = 0.0;
        self.ui_cursor.y = input.window_size.y as f32;
        self.mouse_cursor.x = input.cursor_pos.x as f32;
        self.mouse_cursor.y =
            (input.window_size.y - input.cursor_pos.y) as f32;
        self.window_size.x = input.window_size.x as f32;
        self.window_size.y = input.window_size.y as f32;
    }

    fn advance_rect(&mut self, size: Vec2<f32>) -> Rect {
        use DrawDirection::*;

        let y_step = self.vertical_spacing + size.y;
        let x_step = self.horizontal_spacing + size.x;
        let rect;
        match self.draw_direction {
            Down => {
                rect = Rect::from_top_left(self.ui_cursor, size);
                self.ui_cursor.y -= y_step;
            }
            Up => {
                rect = Rect::from_bot_left(self.ui_cursor, size);
                self.ui_cursor.y += y_step;
            }
            Right => {
                rect = Rect::from_bot_left(self.ui_cursor, size);
                self.ui_cursor.x += x_step;
            }
            Left => {
                rect = Rect::from_bot_right(self.ui_cursor, size);
                self.ui_cursor.x -= x_step;
            }
        }

        rect
    }

    pub fn advance_cursor(&mut self, x: f32, y: f32) {
        self.ui_cursor.x += x;
        self.ui_cursor.y += y;
    }

    pub fn set_draw_direction(&mut self, direction: DrawDirection) {
        self.draw_direction = direction;
    }

    pub fn set_cursor_at_top_left(&mut self) {
        self.ui_cursor.x = 0.0;
        self.ui_cursor.y = self.window_size.y;
    }

    pub fn set_cursor_at_bot_left(&mut self) {
        self.ui_cursor.x = 0.0;
        self.ui_cursor.y = 0.0;
    }

    pub fn set_horizontal_spacing(&mut self, spacing: f32) {
        self.horizontal_spacing = spacing;
    }

    pub fn set_vertical_spacing(&mut self, spacing: f32) {
        self.vertical_spacing = spacing;
    }

    pub fn set_bar_size_scale(
        &mut self,
        width_scale: f32,
        height_scale: f32,
    ) {
        use defaults::*;
        self.bar_size.x = BAR_WIDTH * width_scale;
        self.bar_size.y = BAR_HEIGHT * height_scale;
    }

    pub fn add_bar_size(&mut self, width: f32, height: f32) {
        self.bar_size.x += width;
        self.bar_size.y += height;
    }

    pub fn set_default_bar_size(&mut self) {
        use defaults::*;
        self.bar_size.x = BAR_WIDTH;
        self.bar_size.y = BAR_HEIGHT;
    }

    pub fn reset_horizontal_spacing(&mut self) {
        self.horizontal_spacing = defaults::HORIZONTAL_SPACING;
    }

    pub fn reset_vertical_spacing(&mut self) {
        self.vertical_spacing = defaults::VERTICAL_SPACING;
    }

    pub fn draw(&self, draw_queue: &mut Vec<DrawPrimitive>) {
        draw_queue.extend_from_slice(&self.primitives);
        for text in self.texts.iter() {
            draw_queue.extend_from_slice(&text.get_draw_primitives());
        }
    }

    pub fn rect_with_text(
        &mut self,
        rect_size: Vec2<f32>,
        rect_color: Color,
        font_size: u32,
        string: String,
        text_color: Color,
        glyph_atlas: &GlyphAtlas,
    ) {
        let rect = self.advance_rect(rect_size);
        let primitive = DrawPrimitive::from_rect(
            rect,
            SpaceType::ScreenSpace,
            1.0,
            self.effect,
            rect_color,
        );
        let text = Text::new(
            rect.get_center(),
            glyph_atlas,
            SpaceType::ScreenSpace,
            Origin::Center,
            string,
            font_size,
            text_color,
        );
        self.primitives.push(primitive);
        self.primitives
            .extend_from_slice(&text.get_draw_primitives());
    }

    pub fn bar(&mut self, fill_ratio: f32, color: Color) {
        let size = self.bar_size.mul_x(fill_ratio);
        let rect = self.advance_rect(size);
        let primitive = DrawPrimitive::from_rect(
            rect,
            SpaceType::ScreenSpace,
            1.0,
            self.effect,
            color,
        );
        self.primitives.push(primitive);
    }

    // pub fn button() -> bool {
    //
    // }

    // pub fn text() {
    // }
}
