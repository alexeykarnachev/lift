use crate::entity::*;
use crate::graphics::*;
use crate::vec::*;

pub struct GameOverMenu {
    pub game_over: Entity,
    pub restart: Entity,
    pub quit: Entity,
}

impl GameOverMenu {
    pub fn new(glyph_atlas: &GlyphAtlas) -> Self {
        let mut game_over = Entity::new(Vec2::new(0.0, 200.0));
        game_over.text = Some(Text::from_glyph_atlas(
            &glyph_atlas,
            Space::Screen,
            Origin::Center(Vec2::zeros()),
            "Game Over".to_string(),
            Color::new(1.0, 0.0, 0.0, 1.0),
            2.0,
        ));

        let mut restart = Entity::new(Vec2::new(-50.0, 50.0));
        restart.text = Some(Text::from_glyph_atlas(
            &glyph_atlas,
            Space::Screen,
            Origin::RightCenter(Vec2::zeros()),
            "Restart".to_string(),
            Color::new(1.0, 0.0, 0.0, 1.0),
            1.0,
        ));

        let mut quit = Entity::new(Vec2::new(100.0, 50.0));
        quit.text = Some(Text::from_glyph_atlas(
            &glyph_atlas,
            Space::Screen,
            Origin::LeftCenter(Vec2::zeros()),
            "Quit".to_string(),
            Color::new(1.0, 0.0, 0.0, 1.0),
            1.0,
        ));

        Self {
            game_over,
            restart,
            quit,
        }
    }
}
