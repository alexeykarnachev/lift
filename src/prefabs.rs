use crate::entity::*;
use crate::graphics::*;
use crate::vec::*;

pub fn create_default_sprite_atlas() -> SpriteAtlas {
    SpriteAtlas::new(
        "./assets/sprites/atlas.json",
        "./assets/sprites/atlas.png",
    )
}

pub fn create_floor_entity(idx: usize) -> Entity {
    let size = Vec2::new(100.0, 2.5);
    let y = idx as f32 * size.y;
    let position = Vec2::new(0.0, y);
    let rect = Rect::from_bot_center(Vec2::zeros(), size);
    let primitive =
        DrawPrimitive::with_color(rect, Color::new_gray(0.3, 1.0), 0.0);

    Entity {
        position,
        collider: None,
        kinematic: None,
        health: None,
        weapon: None,
        draw_primitive: Some(primitive),
        animator: None,
    }
}

pub fn create_knight_entity(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    let size = Vec2::new(0.5, 1.0);
    let collider = Rect::from_bot_center(Vec2::zeros(), size);
    let kinematic = Kinematic {
        max_speed: 0.5,
        speed: 0.0,
    };
    let health = Health {
        max: 1000.0,
        current: 1000.0,
    };
    let weapon = Weapon {
        range: 0.5,
        speed: 2.0,
        damage: 10.0,
        cooldown: 0.0,
    };

    let rect = collider;
    let mut animator = Animator::new(
        rect,
        AnimatedSprite::from_atlas(
            sprite_atlas,
            "knight_idle",
            2.0,
            0.025,
        ),
    );

    animator.add(
        "idle",
        AnimatedSprite::from_atlas(
            sprite_atlas,
            "knight_idle",
            0.5,
            0.025,
        ),
    );
    animator.add(
        "attack",
        AnimatedSprite::from_atlas(
            sprite_atlas,
            "knight_attack",
            0.5,
            0.025,
        ),
    );
    animator.add(
        "run",
        AnimatedSprite::from_atlas(sprite_atlas, "knight_run", 0.5, 0.025),
    );

    Entity {
        position,
        collider: Some(collider),
        kinematic: Some(kinematic),
        health: Some(health),
        weapon: Some(weapon),
        draw_primitive: None,
        animator: Some(animator),
    }
}
