use crate::systems::*;
use amethyst::{
    core::{bundle::SystemBundle, Error},
    ecs::prelude::DispatcherBuilder,
};

#[derive(Default)]
pub struct LiveBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for LiveBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(DetectPlayerActionSystem, "detect_player_action", &[]);
        builder.add(
            PositionRenderablesSystem::default(),
            "position_renderable",
            &[],
        );
        Ok(())
    }
}

#[derive(Default)]
pub struct TickBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for TickBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(PlayerMovementSystem, "player_movement", &[]);
        builder.add(ApplyAttacksSystem, "apply_attacks", &["player_movement"]);
        builder.add(ApplyMovementSystem, "apply_movement", &["player_movement"]);
        builder.add(
            AggressiveAISystem,
            "ai_movement",
            &["apply_movement", "apply_attacks"],
        );
        builder.add(ApplyAttacksSystem, "apply_attacks_2", &["ai_movement"]);
        builder.add(ApplyMovementSystem, "apply_movement_2", &["ai_movement"]);
        builder.add(StunSystem, "stun_system", &["apply_attacks_2"]);
        Ok(())
    }
}