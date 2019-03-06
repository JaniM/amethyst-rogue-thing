use crate::{
    data::{Direction, PlayerAction},
    resources::PlayerActionResource,
};
use amethyst::{core::timing::Time, ecs::prelude::*, input::InputHandler};

pub struct DetectPlayerActionSystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    input: Read<'s, InputHandler<String, String>>,
    action: Write<'s, PlayerActionResource>,
    time: Read<'s, Time>,
}

impl<'s> System<'s> for DetectPlayerActionSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let action = if data.input.action_is_down("up").unwrap() {
            Some(PlayerAction::Move(Direction::Up))
        } else if data.input.action_is_down("down").unwrap() {
            Some(PlayerAction::Move(Direction::Down))
        } else if data.input.action_is_down("left").unwrap() {
            Some(PlayerAction::Move(Direction::Left))
        } else if data.input.action_is_down("right").unwrap() {
            Some(PlayerAction::Move(Direction::Right))
        } else if data.input.action_is_down("wait").unwrap() {
            Some(PlayerAction::Wait)
        } else {
            None
        };

        data.action.hold_delay -= data.time.delta_seconds();

        if action.is_some() && data.action.hold_delay <= 0.0 {
            data.action.action = action;
            data.action.hold_delay = 0.25;
        } else if action.is_none() {
            data.action.hold_delay = 0.0;
        }
    }
}
