use crate::entity::*;
use crate::graphics::*;
use crate::ui::*;
use crate::utils::frand;

use crate::vec::*;
use AnimationMode::*;
use EffectType::*;
use Origin::*;

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

pub fn create_default_game_over_ui() -> UI {
    UI::new("./assets/ui/game_over.json")
}

pub fn create_default_play_ui() -> UI {
    UI::new("./assets/ui/play.json")
}

pub fn create_player(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    let max_health = 50000.0;
    let move_speed = 100.0;
    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(20.0, 40.0));
    let weapon_collider = Rect::from_right_center(
        collider.get_right_center(),
        Vec2::new(42.0, 48.0),
    );

    let weapons = vec![Weapon::new(weapon_collider, 0.1, 0.3, 500.0)];
    let dashing = Dashing::new(200.0, 0.5, 0.3);

    let light = Light {
        position: collider.get_top_center(),
        color: Color::new(1.0, 1.0, 1.0, 1.0),
        attenuation: [1.0, 0.05, 0.0],
    };
    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "knight_idle",
        1.2,
        Repeat,
        BotCenter,
    ));
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_idle",
            1.2,
            Repeat,
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
            BotCenter,
        ),
    );

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::Player);
    entity.apply_gravity = true;
    entity.collider = Some(collider);
    entity.move_speed = move_speed;
    entity.max_health = max_health;
    entity.current_health = max_health;
    entity.dashing = Some(dashing);
    entity.weapons = weapons;
    entity.light = Some(light);
    entity.animator = Some(animator);
    entity.effect = ApplyLightEffect as u32;

    entity
}

pub fn create_rat(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    let jump_period = frand(1.8, 2.2);
    let jump_speed = frand(280.0, 320.0);
    let move_speed = frand(30.0, 40.0);
    let max_health = 1000.0;

    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(20.0, 12.0));
    let floor_weapon = Weapon::new(
        Rect::from_right_center(
            collider.get_right_center(),
            Vec2::new(8.0, 12.0),
        ),
        0.5,
        0.3,
        200.0,
    );
    let jump_weapon = Weapon::new(
        Rect::from_center(collider.get_center(), Vec2::new(128.0, 12.0)),
        0.1,
        jump_period,
        200.0,
    );

    let weapons = vec![floor_weapon, jump_weapon];

    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "rat_idle",
        0.5,
        Repeat,
        BotCenter,
    ));
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_idle",
            0.5,
            Repeat,
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
            BotCenter,
        ),
    );
    animator.add(
        "melee_attack",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_melee_attack",
            0.8,
            Once,
            BotCenter,
        ),
    );

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::Rat);
    entity.apply_gravity = true;
    entity.collider = Some(collider);
    entity.move_speed = move_speed;
    entity.jump_speed = jump_speed;
    entity.jump_period = jump_period;
    entity.max_health = max_health;
    entity.current_health = max_health;
    entity.weapons = weapons;
    entity.animator = Some(animator);
    entity.effect = ApplyLightEffect as u32;

    entity
}

pub fn create_bat(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    let max_health = 1000.0;
    let move_speed = frand(60.0, 80.0);
    let healing_speed = frand(80.0, 100.0);
    let healing_duration = frand(4.0, 5.0);
    let healing_cooldown = frand(4.0, 5.0);

    let collider =
        Rect::from_top_center(Vec2::zeros(), Vec2::new(16.0, 16.0));
    let weapon_collider = Rect::from_right_center(
        collider.get_right_center(),
        Vec2::new(4.0, 16.0),
    );

    let weapons = vec![Weapon::new(weapon_collider, 0.2, 0.1, 500.0)];
    let healing =
        Healing::new(healing_speed, healing_duration, healing_cooldown);

    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "bat_wave",
        0.25,
        Repeat,
        TopCenter,
    ));
    animator.add(
        "wave",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_wave",
            0.25,
            Repeat,
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
            TopCenter,
        ),
    );

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::Bat);
    entity.collider = Some(collider);
    entity.move_speed = move_speed;
    entity.max_health = max_health;
    entity.current_health = max_health;
    entity.healing = Some(healing);
    entity.weapons = weapons;
    entity.animator = Some(animator);
    entity.effect = ApplyLightEffect as u32;

    entity
}

pub fn create_torch(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    let animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "torch_burn",
        0.5,
        Repeat,
        TopCenter,
    ));
    let light = Light {
        position,
        color: Color::new(4.0, 0.5, 0.0, 1.0),
        attenuation: [0.05, 0.005, 0.005],
    };

    let mut entity = Entity::new(position);
    entity.light = Some(light);
    entity.animator = Some(animator);

    entity
}

pub fn create_rat_nest(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    let max_health = 20000.0;
    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "rat_nest_idle",
        1.0,
        Repeat,
        TopCenter,
    ));
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_nest_idle",
            1.0,
            Repeat,
            BotCenter,
        ),
    );
    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(60.0, 30.0));
    let spawner = Spawner::new(5.0, 9999, Behaviour::Rat, 50.0, 0.0);

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::Spawner);
    entity.apply_gravity = true;
    entity.collider = Some(collider);
    entity.max_health = max_health;
    entity.current_health = max_health;
    entity.knockback_resist = 1.0;
    entity.animator = Some(animator);
    entity.spawner = Some(spawner);
    entity.effect = ApplyLightEffect as u32;

    entity
}
