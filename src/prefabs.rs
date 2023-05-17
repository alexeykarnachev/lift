use crate::entity::*;
use crate::graphics::*;
use crate::ui::*;
use crate::vec::*;

const FLOOR_WIDTH: f32 = 100.0;
const FLOOR_HEIGHT: f32 = 4.0;
const LIFT_WIDTH: f32 = FLOOR_HEIGHT * 0.6;
const LIFT_HEIGHT: f32 = FLOOR_HEIGHT;
const SHAFT_WIDTH: f32 = LIFT_WIDTH * 1.2;
const SPRITE_SCALE: f32 = 0.05;

pub fn create_default_sprite_atlas() -> SpriteAtlas {
    SpriteAtlas::new(
        "./assets/sprites/atlas.json",
        "./assets/sprites/atlas.png",
    )
}

pub fn create_default_glyph_atlas() -> GlyphAtlas {
    // Typical font sizes: 3, 7, 9, 12, 16, 21, 28, 37, 50, 67, 89, 119, 159
    GlyphAtlas::new(
        "./assets/fonts/Montserrat-Bold.ttf",
        &[16, 28, 67, 119],
    )
}

pub fn create_default_tilemap() -> Tilemap {
    Tilemap::new("./assets/tilemaps/level0.json")
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

pub fn create_floor(
    idx: usize,
    tilemap: &Tilemap,
    sprite_atlas: &SpriteAtlas,
) -> Floor {
    let y = idx as f32 * FLOOR_HEIGHT;
    let primitives = tilemap.get_draw_primitives(
        Origin::BotCenter,
        Vec2::new(0.0, y - 0.5),
        SPRITE_SCALE,
        sprite_atlas,
    );
    Floor::new(y, idx, FLOOR_WIDTH, FLOOR_HEIGHT, primitives)
}

pub fn create_lift_entity(
    floor_idx: usize,
    sprite_atlas: &SpriteAtlas,
) -> Lift {
    let animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "lift",
        1.0,
        AnimationMode::Repeat,
        SPRITE_SCALE,
        Origin::BotCenter,
    ));

    Lift::new(
        floor_idx as f32 * FLOOR_HEIGHT,
        LIFT_WIDTH,
        LIFT_HEIGHT,
        2.0,
        animator,
    )
}

pub fn create_player(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    use AnimationMode::*;
    use Origin::*;

    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(1.0, 2.0));
    let weapon_collider = Rect::from_right_center(
        collider.get_right_center(),
        Vec2::new(2.1, 2.4),
    );

    let melee_weapon = MeleeWeapon::new(weapon_collider, 0.1, 0.22, 500.0);
    let dashing = Dashing::new(9.0, 0.5, 0.3);

    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "knight_idle",
        1.2,
        Repeat,
        SPRITE_SCALE,
        BotCenter,
    ));
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_idle",
            1.2,
            Repeat,
            SPRITE_SCALE,
            BotCenter,
        ),
    );
    animator.add(
        "run",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_run",
            0.8,
            Repeat,
            SPRITE_SCALE,
            BotCenter,
        ),
    );
    animator.add(
        "slide",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_slide",
            0.5,
            Once,
            SPRITE_SCALE,
            BotCenter,
        ),
    );
    animator.add(
        "attack",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_attack",
            0.3,
            Once,
            SPRITE_SCALE,
            BotCenter,
        ),
    );

    Entity::new(
        true,
        Behaviour::Player,
        position,
        true,
        collider,
        4.0,
        6.0,
        0.0,
        5000.0,
        Some(dashing),
        None,
        Some(melee_weapon),
        None,
        Some(animator),
    )
}

pub fn create_rat(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    use AnimationMode::*;
    use Origin::*;

    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(1.0, 0.6));
    let weapon_collider = Rect::from_right_center(
        collider.get_right_center(),
        Vec2::new(0.4, 0.6),
    );

    let melee_weapon = MeleeWeapon::new(weapon_collider, 0.5, 1.0, 500.0);
    let behaviour = Behaviour::Rat {
        min_jump_distance: 2.0,
        max_jump_distance: 3.25,
    };

    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "rat_idle",
        0.5,
        Repeat,
        SPRITE_SCALE,
        BotCenter,
    ));
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_idle",
            0.5,
            Repeat,
            SPRITE_SCALE,
            BotCenter,
        ),
    );
    animator.add(
        "jump",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_jump",
            0.5,
            Once,
            SPRITE_SCALE,
            BotCenter,
        ),
    );
    animator.add(
        "move",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_move",
            0.5,
            Repeat,
            SPRITE_SCALE,
            BotCenter,
        ),
    );
    animator.add(
        "death",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_death",
            0.5,
            Once,
            SPRITE_SCALE,
            BotCenter,
        ),
    );
    animator.add(
        "melee_attack",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_melee_attack",
            0.5,
            Once,
            SPRITE_SCALE,
            BotCenter,
        ),
    );

    Entity::new(
        false,
        behaviour,
        position,
        true,
        collider,
        2.0,
        10.0,
        2.0,
        1000.0,
        None,
        None,
        Some(melee_weapon),
        None,
        Some(animator),
    )
}

pub fn create_bat(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    use AnimationMode::*;
    use Origin::*;

    let collider =
        Rect::from_top_center(Vec2::zeros(), Vec2::new(0.8, 0.8));
    let weapon_collider = Rect::from_right_center(
        collider.get_right_center(),
        Vec2::new(0.2, 0.8),
    );

    let melee_weapon = MeleeWeapon::new(weapon_collider, 0.25, 1.0, 500.0);
    let behaviour = Behaviour::Bat;
    let healing = Healing::new(100.0, 5.0, 5.0);

    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "bat_wave",
        0.25,
        Repeat,
        SPRITE_SCALE,
        TopCenter,
    ));
    animator.add(
        "wave",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_wave",
            0.25,
            Repeat,
            SPRITE_SCALE,
            TopCenter,
        ),
    );
    animator.add(
        "sleep",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_sleep",
            1.0,
            Repeat,
            SPRITE_SCALE,
            TopCenter,
        ),
    );
    animator.add(
        "melee_attack",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_melee_attack",
            0.25,
            Once,
            SPRITE_SCALE,
            TopCenter,
        ),
    );
    animator.add(
        "death",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_death",
            0.5,
            Once,
            SPRITE_SCALE,
            TopCenter,
        ),
    );

    Entity::new(
        false,
        behaviour,
        position,
        false,
        collider,
        2.5,
        8.0,
        2.0,
        1000.0,
        None,
        Some(healing),
        Some(melee_weapon),
        None,
        Some(animator),
    )
}

pub fn create_bat_spawner(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Spawner {
    let entity = create_bat(position, sprite_atlas);

    Spawner::new(position, 5.0, 1, entity)
}

pub fn create_rat_spawner(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Spawner {
    let entity = create_rat(position, sprite_atlas);

    Spawner::new(position, 5.0, 1, entity)
}
