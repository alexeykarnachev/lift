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
    GlyphAtlas::from_ttf("./assets/fonts/Montserrat-Bold.ttf", 62.0)
}

pub fn create_default_game_over_ui(glyph_atlas: &GlyphAtlas) -> UI {
    UI::from_file("./assets/ui/game_over.json", glyph_atlas)
}

pub fn create_shaft_entity(n_floors: usize) -> Entity {
    let height = n_floors as f32 * FLOOR_HEIGHT;
    let size = Vec2::new(SHAFT_WIDTH, height);
    let position = Vec2::new(0.0, 0.0);
    let rect = Rect::from_bot_center(position, size);
    let primitive = DrawPrimitive::from_rect(
        rect,
        Space::World,
        Color::gray(0.05, 1.0),
    );

    let mut entity = Entity::new(position);
    entity.draw_primitive = Some(primitive);

    entity
}

pub fn create_floor_entity(idx: usize) -> Entity {
    let size = Vec2::new(FLOOR_WIDTH, FLOOR_HEIGHT);
    let y = idx as f32 * FLOOR_HEIGHT;
    let position = Vec2::new(0.0, y);
    let rect = Rect::from_bot_center(Vec2::zeros(), size);
    let primitive = DrawPrimitive::from_rect(
        rect,
        Space::World,
        Color::gray(0.3, 1.0),
    );

    let mut entity = Entity::new(position);
    entity.draw_primitive = Some(primitive);

    entity
}

pub fn create_lift_entity(floor_idx: usize) -> Entity {
    let size = Vec2::new(LIFT_WIDTH, LIFT_HEIGHT);
    let y = floor_idx as f32 * FLOOR_HEIGHT;
    let position = Vec2::new(0.0, y);
    let rect = Rect::from_bot_center(Vec2::zeros(), size);
    let primitive = DrawPrimitive::from_rect(
        rect,
        Space::World,
        Color::gray(0.6, 1.0),
    );
    let kinematic = Kinematic {
        max_speed: 2.0,
        speed: 0.0,
        target: None,
    };

    let mut entity = Entity::new(position);
    entity.kinematic = Some(kinematic);
    entity.draw_primitive = Some(primitive);

    entity
}

pub fn create_destroyer_entity(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(0.5, 1.0));
    let kinematic = Kinematic::new(1.0);
    let health = Health::new(200.0);
    let weapon = Weapon::new(2.0, 2.0, 100.0);

    let rect = collider;
    let scale = 0.015;
    let mut animator = Animator::new(
        rect,
        AnimatedSprite::from_sprite_atlas(
            sprite_atlas,
            "destroyer_idle",
            AnimationMode::Repeat,
            scale,
        ),
    );

    use AnimationType::*;
    animator.add(
        Idle,
        AnimatedSprite::from_sprite_atlas(
            sprite_atlas,
            "destroyer_idle",
            AnimationMode::Repeat,
            scale,
        ),
    );
    animator.add(
        Attack,
        AnimatedSprite::from_sprite_atlas(
            sprite_atlas,
            "destroyer_shoot",
            AnimationMode::RepeatFrom(4),
            scale,
        ),
    );
    animator.add(
        Move,
        AnimatedSprite::from_sprite_atlas(
            sprite_atlas,
            "destroyer_walk",
            AnimationMode::Repeat,
            scale,
        ),
    );
    animator.add(
        Hurt,
        AnimatedSprite::from_sprite_atlas(
            sprite_atlas,
            "destroyer_hurt",
            AnimationMode::Once,
            scale,
        ),
    );
    animator.add(
        Die,
        AnimatedSprite::from_sprite_atlas(
            sprite_atlas,
            "destroyer_dead",
            AnimationMode::Once,
            scale,
        ),
    );

    let mut entity = Entity::new(position);
    entity.collider = Some(collider);
    entity.kinematic = Some(kinematic);
    entity.health = Some(health);
    entity.weapon = Some(weapon);
    entity.animator = Some(animator);

    entity
}
