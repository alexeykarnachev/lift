use crate::frame::*;
use crate::input::*;
use crate::renderer::*;
use crate::vec::*;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum State {
    Idle,
    Attack0,
    Attack1,
    Attack2,
}

pub struct Knight {
    curr_state: State,
    next_state: State,

    animators: HashMap<State, FrameAnimator>,
}

impl Knight {
    pub fn new(frame_atlas: &FrameAtlas) -> Self {
        use State::*;
        let animators = HashMap::from([
            (Idle, frame_atlas.get_animator("knight_idle", 0.1, true)),
            (
                Attack0,
                frame_atlas.get_animator("knight_attack_0", 0.1, false),
            ),
            (
                Attack1,
                frame_atlas.get_animator("knight_attack_1", 0.1, false),
            ),
            (
                Attack2,
                frame_atlas.get_animator("knight_attack_2", 0.1, false),
            ),
        ]);

        Self {
            curr_state: Idle,
            next_state: Idle,
            animators,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        input: &Input,
        renderer: &mut Renderer,
    ) {
        use State::*;
        if input.is_action(Keyaction::Attack) {
            match self.curr_state {
                Idle => {
                    self.curr_state = Attack0;
                    self.next_state = Idle;
                }
                Attack0 => {
                    self.next_state = Attack1;
                }
                Attack1 => {
                    self.next_state = Attack2;
                }
                _ => {}
            }
        } else {
            self.next_state = Idle;
        }

        let animator = self.get_animator();
        let frame = animator.update(dt);
        let primitive = DrawPrimitive::world_sprite(
            frame.sprite,
            Pivot::BotCenter(Vec2::zeros()),
            false,
            false,
        );
        renderer.push_primitive(primitive);

        if animator.is_finished() {
            self.curr_state = self.next_state;
            self.next_state = Idle;
        }
    }

    fn get_animator(&mut self) -> &mut FrameAnimator {
        self.animators.get_mut(&self.curr_state).unwrap()
    }
}
