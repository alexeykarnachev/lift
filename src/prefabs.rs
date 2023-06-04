use crate::entity::*;
use crate::graphics::*;
use crate::player_stats::Stats;
use crate::utils::frand;
use std::f32::consts::PI;

use crate::vec::*;
// use AnimationMode::*;
use EffectType::*;
// use Origin::*;

mod player {
    pub const MAX_HEALTH: f32 = 2000.0 * 1000.0;
    pub const MOVE_SPEED: f32 = 100.0;
    pub const KNOCKBACK_RESIST: f32 = 100.0;

    pub mod stamina {
        pub const MAX: f32 = 80000.0;
        pub const REGEN: f32 = 5000.0;
    }

    pub mod weapon {
        pub const ANTICIPATION_TIME: f32 = 0.1;
        pub const ACTION_TIME: f32 = 2.0;
        pub const DAMAGE: f32 = 800.0;
        pub const KNOCKBACK: f32 = 120.0;
        pub const STAMINA_COST: f32 = 10000.0;
    }

    pub mod dashing {
        pub const SPEED: f32 = 150.0;
        pub const STAMINA_COST: f32 = 15000.0;
        pub const ACTION_TIME: f32 = 0.8;
        pub const RECOVERY_TIME: f32 = 0.2;
        pub const COOLDOWN_TIME: f32 = 0.3;
    }

    pub mod animator {
        pub const IDLE_DURATION: f32 = 1.2;
        pub const RUN_DURATION: f32 = 0.8;
        pub const ROLL_DURATION: f32 = 1.0;
        pub const CLIMB_DURATION: f32 = 1.0;
        pub const ATTACK_DURATION: f32 = 2.1;
    }
}

mod rat {
    pub const MAX_HEALTH: f32 = 1000.0;
    pub const VIEW_DISTANCE: f32 = 300.0;
    pub const MOVE_SPEED_RANGE: (f32, f32) = (30.0, 40.0);
    pub const KNOCKBACK_RESIST: f32 = 20.0;
    pub const EXP_DROP: usize = 25;

    pub mod jumping {
        pub const COOLDOWN_TIME: f32 = 3.0;
        pub const SPEED_RANGE: (f32, f32) = (280.0, 320.0);
    }

    pub mod floor_weapon {
        pub const ANTICIPATION_TIME: f32 = 0.3;
        pub const ACTION_TIME: f32 = 0.3;
        pub const DAMAGE: f32 = 200.0;
    }

    pub mod jump_weapon {
        pub const COOLDOWN_TIME: f32 = 2.0;
        pub const DAMAGE: f32 = 200.0;
    }

    pub mod animator {
        pub const IDLE_DURATION: f32 = 0.5;
        pub const JUMP_DURATION: f32 = 0.5;
        pub const MOVE_DURATION: f32 = 0.5;
        pub const DEATH_DURATION: f32 = 0.5;
        pub const MELEE_ATTACK_DURATION: f32 = 0.6;
    }
}

mod bat {
    pub const MAX_HEALTH: f32 = 1000.0;
    pub const VIEW_DISTANCE: f32 = 300.0;
    pub const MOVE_SPEED_RANGE: (f32, f32) = (60.0, 80.0);
    pub const HEALING_SPEED_RANGE: (f32, f32) = (80.0, 100.0);
    pub const HEALING_DURATION_TIME_RANGE: (f32, f32) = (4.0, 5.0);
    pub const HEALING_COOLDOWN_TIME_RANGE: (f32, f32) = (4.0, 5.0);
    pub const KNOCKBACK_RESIST: f32 = 0.0;
    pub const EXP_DROP: usize = 15;

    pub mod weapon {
        pub const ANTICIPATION_TIME: f32 = 0.2;
        pub const ACTION_TIME: f32 = 0.1;
        pub const DAMAGE: f32 = 500.0;
    }

    pub mod animator {
        pub const WAVE_DURATION: f32 = 0.25;
        pub const SLEEP_DURATION: f32 = 1.0;
        pub const DEATH_DURATION: f32 = 0.5;
        pub const MELEE_ATTACK_DURATION: f32 = 0.25;
    }
}

mod rat_king {
    pub const MAX_HEALTH: f32 = 10000.0;
    pub const VIEW_DISTANCE: f32 = 300.0;
    pub const MOVE_SPEED: f32 = 50.0;
    pub const KNOCKBACK_RESIST: f32 = 9999.0;
    pub const EXP_DROP: usize = 50;

    pub mod dashing {
        pub const SPEED: f32 = 300.0;
        pub const ANTICIPATION_TIME: f32 = 0.5;
        pub const ACTION_TIME: f32 = 0.5;
        pub const COOLDOWN_TIME: f32 = 5.0;
    }

    pub mod floor_weapon {
        pub const ANTICIPATION_TIME: f32 = 0.2;
        pub const ACTION_TIME: f32 = 0.4;
        pub const DAMAGE: f32 = 200.0;
    }

    pub mod roll_weapon {
        pub const COOLDOWN_TIME: f32 = 3.0;
        pub const DAMAGE: f32 = 2000.0;
        pub const KNOCKBACK: f32 = 400.0;
    }

    pub mod animator {
        pub const RISE_DURATION: f32 = 0.6;
        pub const IDLE_DURATION: f32 = 0.5;
        pub const MOVE_DURATION: f32 = 0.5;
        pub const MELEE_ATTACK_DURATION: f32 = 0.6;
        pub const ROLL_DURATION: f32 = 0.3;
        pub const DEATH_DURATION: f32 = 1.2;
    }
}

mod rat_nest {
    pub const MAX_HEALTH: f32 = 10000.0;
    pub const EXP_DROP: usize = 35;

    pub mod spawner {
        pub const SPAWN_PERIOD: f32 = 5.0;
        pub const N_ALIVE_MAX: u32 = 3;
        pub const SPAWN_RANGE_X: f32 = 50.0;
    }

    pub mod animator {
        pub const IDLE_DURATION: f32 = 1.0;
        pub const DEATH_DURATION: f32 = 0.8;
    }
}

mod torch {
    pub mod animator {
        pub const BURN_DURATION: f32 = 0.5;
    }
}

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
        &[16, 28, 37, 67, 119],
    )
}

pub fn create_player(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    use player::*;

    let stamina = Stamina::new(stamina::MAX, stamina::REGEN);
    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(20.0, 40.0));
    let weapon_collider = Rect::from_right_center(
        collider.get_right_center(),
        Vec2::new(42.0, 48.0),
    );

    let weapons = vec![Weapon::new(
        weapon_collider,
        weapon::DAMAGE,
        weapon::KNOCKBACK,
        weapon::STAMINA_COST,
        AbilityTimer::new(
            weapon::ANTICIPATION_TIME,
            weapon::ACTION_TIME,
            0.0,
            0.0,
        ),
    )];
    let dashing = Dashing::new(
        dashing::SPEED,
        dashing::STAMINA_COST,
        AbilityTimer::new(
            0.0,
            dashing::ACTION_TIME,
            dashing::RECOVERY_TIME,
            dashing::COOLDOWN_TIME,
        ),
    );

    let light = Light::player();
    /*
    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "knight_idle",
        animator::IDLE_DURATION,
        Repeat,
        BotCenter,
    ));
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_idle",
            animator::IDLE_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "walk",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_walk",
            animator::RUN_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "roll",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_roll",
            animator::ROLL_DURATION,
            Once,
            BotCenter,
        ),
    );
    animator.add(
        "climb",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_climb",
            animator::CLIMB_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "attack",
        AnimatedSprite::new(
            sprite_atlas,
            "knight_attack",
            animator::ATTACK_DURATION,
            Once,
            BotCenter,
        ),
    );
    */

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::Player);
    entity.knockback_resist = KNOCKBACK_RESIST;
    entity.apply_gravity = true;
    entity.collider = Some(collider);
    entity.move_speed = player::MOVE_SPEED;
    entity.max_health = player::MAX_HEALTH;
    entity.current_health = player::MAX_HEALTH;
    entity.stamina = Some(stamina);
    entity.dashing = Some(dashing);
    entity.weapons = weapons;
    entity.light = Some(light);
    // entity.animator = Some(animator);
    entity.effect = ApplyLightEffect as u32;
    entity.stats = Some(Stats::new());

    entity
}

pub fn create_rat(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    use rat::*;

    let move_speed = frand(MOVE_SPEED_RANGE.0, MOVE_SPEED_RANGE.1);
    let jump_speed = frand(jumping::SPEED_RANGE.0, jumping::SPEED_RANGE.1);
    let jumping = Jumping::new(
        jump_speed,
        0.0,
        PI * 0.1,
        AbilityTimer::new(0.0, 0.5, 0.0, jumping::COOLDOWN_TIME),
    );

    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(20.0, 12.0));
    let floor_weapon = Weapon::new(
        Rect::from_center(collider.get_center(), Vec2::new(50.0, 12.0)),
        floor_weapon::DAMAGE,
        0.0,
        0.0,
        AbilityTimer::new(
            floor_weapon::ANTICIPATION_TIME,
            floor_weapon::ACTION_TIME,
            0.0,
            0.0,
        ),
    );
    let jump_weapon = Weapon::new(
        Rect::from_left_center(
            collider.get_center(),
            Vec2::new(50.0, 12.0),
        ),
        jump_weapon::DAMAGE,
        0.0,
        0.0,
        AbilityTimer::new(0.0, 0.0, 0.0, jump_weapon::COOLDOWN_TIME),
    );

    let weapons = vec![floor_weapon, jump_weapon];

    /*
    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "rat_idle",
        animator::IDLE_DURATION,
        Repeat,
        BotCenter,
    ));
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_idle",
            animator::IDLE_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "jump",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_jump",
            animator::JUMP_DURATION,
            Once,
            BotCenter,
        ),
    );
    animator.add(
        "move",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_move",
            animator::MOVE_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "death",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_death",
            animator::DEATH_DURATION,
            Once,
            BotCenter,
        ),
    );
    animator.add(
        "melee_attack",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_melee_attack",
            animator::MELEE_ATTACK_DURATION,
            Once,
            BotCenter,
        ),
    );
    */

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::Rat);
    entity.apply_gravity = true;
    entity.collider = Some(collider);
    entity.view_distance = VIEW_DISTANCE;
    entity.move_speed = move_speed;
    entity.jumping = Some(jumping);
    entity.max_health = MAX_HEALTH;
    entity.current_health = MAX_HEALTH;
    entity.knockback_resist = KNOCKBACK_RESIST;
    entity.weapons = weapons;
    // entity.animator = Some(animator);
    entity.effect = ApplyLightEffect as u32;
    entity.exp_drop = EXP_DROP;

    entity
}

pub fn create_bat(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    use bat::*;

    let move_speed = frand(MOVE_SPEED_RANGE.0, MOVE_SPEED_RANGE.1);
    let healing_speed =
        frand(HEALING_SPEED_RANGE.0, HEALING_SPEED_RANGE.1);
    let healing_action_time = frand(
        HEALING_DURATION_TIME_RANGE.0,
        HEALING_DURATION_TIME_RANGE.1,
    );
    let healing_cooldown_time = frand(
        HEALING_COOLDOWN_TIME_RANGE.0,
        HEALING_COOLDOWN_TIME_RANGE.1,
    );

    let collider =
        Rect::from_top_center(Vec2::zeros(), Vec2::new(16.0, 16.0));
    let weapon_collider =
        Rect::from_center(collider.get_center(), Vec2::new(20.0, 16.0));

    let weapons = vec![Weapon::new(
        weapon_collider,
        weapon::DAMAGE,
        0.0,
        0.0,
        AbilityTimer::new(
            weapon::ANTICIPATION_TIME,
            weapon::ACTION_TIME,
            0.0,
            0.0,
        ),
    )];
    let healing = Healing::new(
        healing_speed,
        AbilityTimer::new(
            0.0,
            healing_action_time,
            0.0,
            healing_cooldown_time,
        ),
    );

    /*
    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "bat_wave",
        animator::WAVE_DURATION,
        Repeat,
        TopCenter,
    ));
    animator.add(
        "wave",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_wave",
            animator::WAVE_DURATION,
            Repeat,
            TopCenter,
        ),
    );
    animator.add(
        "sleep",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_sleep",
            animator::SLEEP_DURATION,
            Repeat,
            TopCenter,
        ),
    );
    animator.add(
        "melee_attack",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_melee_attack",
            animator::MELEE_ATTACK_DURATION,
            Once,
            TopCenter,
        ),
    );
    animator.add(
        "death",
        AnimatedSprite::new(
            sprite_atlas,
            "bat_death",
            animator::DEATH_DURATION,
            Once,
            TopCenter,
        ),
    );
    */

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::Bat);
    entity.collider = Some(collider);
    entity.view_distance = VIEW_DISTANCE;
    entity.move_speed = move_speed;
    entity.max_health = MAX_HEALTH;
    entity.current_health = MAX_HEALTH;
    entity.knockback_resist = KNOCKBACK_RESIST;
    entity.healing = Some(healing);
    entity.weapons = weapons;
    // entity.animator = Some(animator);
    entity.effect = ApplyLightEffect as u32;
    entity.exp_drop = EXP_DROP;

    entity
}

pub fn create_rat_king(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    use rat_king::*;

    let dashing = Dashing::new(
        dashing::SPEED,
        0.0,
        AbilityTimer::new(
            dashing::ANTICIPATION_TIME,
            dashing::ACTION_TIME,
            0.0,
            dashing::COOLDOWN_TIME,
        ),
    );

    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(50.0, 40.0));
    let floor_weapon = Weapon::new(
        Rect::from_center(collider.get_center(), Vec2::new(80.0, 20.0)),
        floor_weapon::DAMAGE,
        0.0,
        0.0,
        AbilityTimer::new(
            floor_weapon::ANTICIPATION_TIME,
            floor_weapon::ACTION_TIME,
            0.0,
            0.0,
        ),
    );
    let roll_weapon = Weapon::new(
        Rect::from_center(collider.get_center(), Vec2::new(60.0, 12.0)),
        roll_weapon::DAMAGE,
        roll_weapon::KNOCKBACK,
        0.0,
        AbilityTimer::new(0.0, 0.0, 0.0, roll_weapon::COOLDOWN_TIME),
    );
    let weapons = vec![floor_weapon, roll_weapon];

    /*
    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "rat_king_rise",
        animator::RISE_DURATION,
        Once,
        BotCenter,
    ));
    animator.add(
        "rise",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_king_rise",
            animator::RISE_DURATION,
            Once,
            BotCenter,
        ),
    );
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_king_idle",
            animator::IDLE_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "move",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_king_move",
            animator::MOVE_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "melee_attack",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_king_melee_attack",
            animator::MELEE_ATTACK_DURATION,
            Once,
            BotCenter,
        ),
    );
    animator.add(
        "roll",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_king_roll",
            animator::ROLL_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "death",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_king_death",
            animator::DEATH_DURATION,
            Once,
            BotCenter,
        ),
    );
    */

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::RatKing);
    entity.apply_gravity = true;
    entity.collider = Some(collider);
    entity.view_distance = VIEW_DISTANCE;
    entity.move_speed = MOVE_SPEED;
    entity.dashing = Some(dashing);
    entity.max_health = MAX_HEALTH;
    entity.current_health = MAX_HEALTH;
    entity.knockback_resist = KNOCKBACK_RESIST;
    entity.weapons = weapons;
    // entity.animator = Some(animator);
    entity.effect = ApplyLightEffect as u32;
    entity.exp_drop = EXP_DROP;

    entity
}

pub fn create_rat_nest(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    use rat_nest::*;

    let collider =
        Rect::from_bot_center(Vec2::zeros(), Vec2::new(60.0, 30.0));
    let spawner = Spawner::new(
        spawner::SPAWN_PERIOD,
        9999,
        spawner::N_ALIVE_MAX,
        Behaviour::Rat,
        spawner::SPAWN_RANGE_X,
        0.0,
    );

    /*
    let mut animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "rat_nest_idle",
        animator::IDLE_DURATION,
        Repeat,
        TopCenter,
    ));
    animator.add(
        "idle",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_nest_idle",
            animator::IDLE_DURATION,
            Repeat,
            BotCenter,
        ),
    );
    animator.add(
        "death",
        AnimatedSprite::new(
            sprite_atlas,
            "rat_nest_death",
            animator::DEATH_DURATION,
            Once,
            BotCenter,
        ),
    );
    */

    let mut entity = Entity::new(position);
    entity.behaviour = Some(Behaviour::RatNest);
    entity.apply_gravity = true;
    entity.collider = Some(collider);
    entity.max_health = MAX_HEALTH;
    entity.current_health = MAX_HEALTH;
    entity.knockback_resist = 9999.0;
    // entity.animator = Some(animator);
    entity.spawner = Some(spawner);
    entity.effect = ApplyLightEffect as u32;
    entity.exp_drop = EXP_DROP;

    entity
}

pub fn create_torch(
    position: Vec2<f32>,
    sprite_atlas: &SpriteAtlas,
) -> Entity {
    // use torch::*;

    /*
    let animator = Animator::new(AnimatedSprite::new(
        sprite_atlas,
        "torch_burn",
        animator::BURN_DURATION,
        Repeat,
        TopCenter,
    ));
    */
    let light = Light::torch();

    let mut entity = Entity::new(position);
    entity.light = Some(light);
    // entity.animator = Some(animator);
    entity.particles_emitter =
        ParticlesEmitter::torch(Vec2::new(0.0, 4.0));

    entity
}

pub fn create_stone_wall(rect: Rect) -> DrawPrimitive {
    use EffectType::*;
    let effect = ApplyLightEffect as u32 | StoneWallEffect as u32;
    DrawPrimitive::from_rect(
        rect,
        SpaceType::WorldSpace,
        -1.0,
        effect,
        Color::gray(0.6, 1.0),
    )
}
