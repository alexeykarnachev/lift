use crate::entity::*;
use crate::graphics::*;
use crate::ui::*;
use crate::vec::*;

const FLOOR_WIDTH: f32 = 100.0;
const FLOOR_HEIGHT: f32 = 2.5;
const LIFT_WIDTH: f32 = FLOOR_HEIGHT * 0.6;
const LIFT_HEIGHT: f32 = FLOOR_HEIGHT;
const SHAFT_WIDTH: f32 = LIFT_WIDTH * 1.2;

pub fn create_default_sprite_atlas() -> SpriteAtlas {
    SpriteAtlas::from_image(
        "./assets/sprites/atlas.json",
        "./assets/sprites/atlas.png",
    )
}

pub fn create_default_glyph_atlas() -> GlyphAtlas {
    // Typical font sizes: 3, 7, 9, 12, 16, 21, 28, 37, 50, 67, 89, 119, 159
    GlyphAtlas::from_ttf(
        "./assets/fonts/Montserrat-Bold.ttf",
        &[16, 28, 67, 119],
    )
}

pub fn create_default_game_over_ui() -> UI {
    UI::new("./assets/ui/game_over.json")
}

pub fn create_default_play_ui() -> UI {
    UI::new("./assets/ui/play.json")
}

pub fn create_shaft(n_floors: usize) -> Shaft {
    Shaft::new(SHAFT_WIDTH, n_floors as f32 * FLOOR_HEIGHT)
}

pub fn create_floor(idx: usize) -> Floor {
    Floor::new(idx as f32 * FLOOR_HEIGHT, idx, FLOOR_WIDTH, FLOOR_HEIGHT)
}

pub fn create_lift_entity(floor_idx: usize) -> Lift {
    Lift::new(
        floor_idx as f32 * FLOOR_HEIGHT,
        LIFT_WIDTH,
        LIFT_HEIGHT,
        2.0,
    )
}

pub fn create_player(position: Vec2<f32>) -> Entity {
    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(0.5, 1.0));
    let range_weapon =
        RangeWeapon::new(Vec2::new(0.0, 0.7), 0.5, 0.5, 30.0, 100.0);

    Entity::new(
        true,
        Behaviour::Player,
        position,
        collider,
        4.0,
        5.0,
        0.0,
        5000.0,
        None,
        Some(range_weapon),
    )
}

pub fn create_rat(position: Vec2<f32>) -> Entity {
    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(0.3, 0.2));
    let melee_weapon =
        MeleeWeapon::new(Vec2::new(0.0, 0.1), 1.0, 1.0, 100.0);

    Entity::new(
        false,
        Behaviour::Rat,
        position,
        collider,
        2.0,
        8.0,
        2.0,
        1000.0,
        Some(melee_weapon),
        None,
    )
}

pub fn create_rat_spawner(position: Vec2<f32>) -> Spawner {
    let entity = create_rat(position);

    Spawner::new(position, 5.0, 1, entity)
}
