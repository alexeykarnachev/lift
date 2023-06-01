use crate::entity::Text;
use crate::graphics::*;
use crate::input::*;
use crate::vec::*;

mod defaults {
    pub const VERTICAL_PADDING: f32 = 10.0;
    pub const HORIZONTAL_PADDING: f32 = 10.0;

    pub const FONT_SIZE: u32 = 28;
    pub const TEXT_RGBA: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

    pub const BAR_WIDTH: f32 = 250.0;
    pub const BAR_HEIGHT: f32 = 20.0;

    pub const BUTTON_WIDTH: f32 = 200.0;
    pub const BUTTON_HEIGHT: f32 = 50.0;
    pub const BUTTON_COLD_RGBA: [f32; 4] = [0.6, 0.6, 0.6, 1.0];
    pub const BUTTON_HOT_RGBA: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
    pub const BUTTON_ACTIVE_RGBA: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    pub const SPRITE_COLD_ALPHA: f32 = 0.5;
    pub const SPRITE_HOT_ALPHA: f32 = 0.8;
    pub const SPRITE_ACTIVE_ALPHA: f32 = 1.0;
    pub const SPRITE_SCALE: f32 = 4.0;

    pub const GROUP_BACKGROUND_RGBA: [f32; 4] = [0.05, 0.05, 0.05, 1.0];
    pub const GROUP_BACKGROUND_PADDING: f32 = 50.0;
}

#[derive(PartialEq)]
enum ButtonState {
    Cold,
    Hot,
    Active,
    Released,
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
    lmb_is_just_up: bool,
    lmb_is_down: bool,

    draw_direction: DrawDirection,

    vertical_padding: f32,
    horizontal_padding: f32,

    font_size: u32,
    text_color: Color,

    button_size: Vec2<f32>,
    button_cold_color: Color,
    button_hot_color: Color,
    button_active_color: Color,

    sprite_cold_alpha: f32,
    sprite_hot_alpha: f32,
    sprite_active_alpha: f32,
    sprite_scale: f32,

    group_background_color: Color,
    group_background_padding: f32,

    bar_size: Vec2<f32>,

    effect: u32,
    is_group_started: bool,
    group_rect: Option<Rect>,

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

    pub fn begin(&mut self, input: &Input) {
        self.primitives.clear();
        self.texts.clear();

        use defaults::*;
        self.draw_direction = DrawDirection::Down;
        self.vertical_padding = VERTICAL_PADDING;
        self.horizontal_padding = HORIZONTAL_PADDING;
        self.font_size = FONT_SIZE;
        self.text_color = Color::from_slice(&TEXT_RGBA);
        self.button_size = Vec2::new(BUTTON_WIDTH, BUTTON_HEIGHT);
        self.button_cold_color = Color::from_slice(&BUTTON_COLD_RGBA);
        self.button_hot_color = Color::from_slice(&BUTTON_HOT_RGBA);
        self.button_active_color = Color::from_slice(&BUTTON_ACTIVE_RGBA);
        self.sprite_cold_alpha = SPRITE_COLD_ALPHA;
        self.sprite_hot_alpha = SPRITE_HOT_ALPHA;
        self.sprite_active_alpha = SPRITE_ACTIVE_ALPHA;
        self.sprite_scale = SPRITE_SCALE;
        self.group_background_color =
            Color::from_slice(&GROUP_BACKGROUND_RGBA);
        self.group_background_padding = GROUP_BACKGROUND_PADDING;
        self.bar_size = Vec2::new(BAR_WIDTH, BAR_HEIGHT);
        self.effect = 0;
        self.is_group_started = false;

        self.ui_cursor.x = 0.0;
        self.ui_cursor.y = input.window_size.y as f32;
        self.mouse_cursor.x = input.cursor_pos.x as f32;
        self.mouse_cursor.y =
            (input.window_size.y - input.cursor_pos.y) as f32;
        self.window_size.x = input.window_size.x as f32;
        self.window_size.y = input.window_size.y as f32;
        self.lmb_is_just_up = input.lmb_is_just_up;
        self.lmb_is_down = input.lmb_is_down;
    }

    pub fn get_ui_cursor(&self) -> Vec2<f32> {
        self.ui_cursor
    }

    fn advance_rect(&mut self, size: Vec2<f32>) -> Rect {
        use DrawDirection::*;

        let y_step = self.vertical_padding + size.y;
        let x_step = self.horizontal_padding + size.x;
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

        if self.is_group_started {
            self.group_rect = if let Some(group_rect) = self.group_rect {
                Some(group_rect.merge(rect))
            } else {
                Some(rect)
            };
        }

        rect
    }

    pub fn start_group(&mut self) {
        self.is_group_started = true;
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

    pub fn set_cursor_at(&mut self, position: Vec2<f32>) {
        self.ui_cursor = position;
    }

    pub fn set_horizontal_padding(&mut self, padding: f32) {
        self.horizontal_padding = padding;
    }

    pub fn set_vertical_padding(&mut self, padding: f32) {
        self.vertical_padding = padding;
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

    pub fn set_font_size(&mut self, font_size: u32) {
        self.font_size = font_size;
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

    pub fn reset_horizontal_padding(&mut self) {
        self.horizontal_padding = defaults::HORIZONTAL_PADDING;
    }

    pub fn reset_vertical_padding(&mut self) {
        self.vertical_padding = defaults::VERTICAL_PADDING;
    }

    pub fn reset_font_size(&mut self) {
        self.font_size = defaults::FONT_SIZE;
    }

    pub fn draw(&self, draw_queue: &mut Vec<DrawPrimitive>) {
        draw_queue.extend_from_slice(&self.primitives);
        for text in self.texts.iter() {
            draw_queue.extend_from_slice(&text.get_draw_primitives());
        }
    }

    pub fn rect_button(
        &mut self,
        string: &str,
        glyph_atlas: &GlyphAtlas,
    ) -> bool {
        use ButtonState::*;

        let rect = self.advance_rect(self.button_size);
        let text = Text::new(
            rect.get_center(),
            glyph_atlas,
            SpaceType::ScreenSpace,
            Origin::Center,
            string.to_string(),
            self.font_size,
            self.text_color,
        );

        let state = self.get_button_state(rect);
        let color = match state {
            Cold => self.button_cold_color,
            Hot => self.button_hot_color,
            _ => self.button_active_color,
        };
        let primitive = DrawPrimitive::from_rect(
            rect,
            SpaceType::ScreenSpace,
            1.0,
            self.effect,
            color,
        );

        self.primitives.push(primitive);
        self.primitives
            .extend_from_slice(&text.get_draw_primitives());

        state == ButtonState::Released
    }

    pub fn text_button(
        &mut self,
        string: &str,
        glyph_atlas: &GlyphAtlas,
    ) -> bool {
        use ButtonState::*;

        let mut text = Text::new(
            Vec2::zeros(),
            glyph_atlas,
            SpaceType::ScreenSpace,
            Origin::BotLeft,
            string.to_string(),
            self.font_size,
            self.text_color,
        );

        let rect_size = text
            .get_bound_rect()
            .get_size()
            .with_y(self.font_size as f32);
        let rect = self.advance_rect(rect_size);
        let state = self.get_button_state(rect);
        let color = match state {
            Cold => self.button_cold_color,
            Hot => self.button_hot_color,
            _ => self.button_active_color,
        };

        text.set_color(color);
        text.set_position(rect.bot_left);

        self.primitives
            .extend_from_slice(&text.get_draw_primitives());

        state == ButtonState::Released
    }

    pub fn sprite_button(&mut self, sprite: Sprite) -> bool {
        use ButtonState::*;

        let mut primitive = DrawPrimitive::from_sprite(
            SpaceType::ScreenSpace,
            1.0,
            0,
            Vec2::zeros(),
            sprite,
            None,
            false,
            TextureType::SpriteTexture,
            self.sprite_scale,
        );
        let rect = self.advance_rect(primitive.rect.get_size());
        let state = self.get_button_state(rect);
        let alpha = match state {
            Cold => self.sprite_cold_alpha,
            Hot => self.sprite_hot_alpha,
            _ => self.sprite_active_alpha,
        };
        primitive.rect = rect;
        primitive.color = Some(Color::only_alpha(alpha));

        self.primitives.push(primitive);

        state == ButtonState::Released
    }

    pub fn sprite(&mut self, sprite: Sprite, alpha: f32) {
        let mut primitive = DrawPrimitive::from_sprite(
            SpaceType::ScreenSpace,
            1.0,
            0,
            Vec2::zeros(),
            sprite,
            Some(Color::only_alpha(alpha)),
            false,
            TextureType::SpriteTexture,
            self.sprite_scale,
        );
        let rect = self.advance_rect(primitive.rect.get_size());
        primitive.rect = rect;

        self.primitives.push(primitive);
    }

    fn get_button_state(&self, rect: Rect) -> ButtonState {
        use ButtonState::*;
        let state;
        if rect.collide_with_point(self.mouse_cursor) {
            if self.lmb_is_down {
                state = Active;
            } else if self.lmb_is_just_up {
                state = Released;
            } else {
                state = Hot
            }
        } else {
            state = Cold;
        };

        state
    }

    pub fn text(&mut self, string: &str, glyph_atlas: &GlyphAtlas) {
        let mut text = Text::new(
            Vec2::zeros(),
            glyph_atlas,
            SpaceType::ScreenSpace,
            Origin::BotLeft,
            string.to_string(),
            self.font_size,
            self.text_color,
        );

        let rect_size = text
            .get_bound_rect()
            .get_size()
            .with_y(self.font_size as f32);
        let rect = self.advance_rect(rect_size);
        text.set_position(rect.bot_left);

        self.primitives
            .extend_from_slice(&text.get_draw_primitives());
    }

    pub fn rect_with_text(
        &mut self,
        rect_size: Vec2<f32>,
        rect_color: Color,
        font_size: u32,
        string: &str,
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
            string.to_string(),
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

    pub fn group_background(&mut self) {
        if !self.is_group_started {
            return;
        }

        self.is_group_started = false;
        if let Some(rect) = self.group_rect {
            let rect = rect.expand_from_center(
                self.group_background_padding,
                self.group_background_padding,
            );
            let primitive = DrawPrimitive::from_rect(
                rect,
                SpaceType::ScreenSpace,
                0.9,
                0,
                self.group_background_color,
            );
            self.primitives.push(primitive);
        }
    }
}
