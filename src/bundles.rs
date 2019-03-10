use crate::{systems::*, tui::TuiBundle};
use amethyst::{
    core::{bundle::SystemBundle, Error},
    ecs::prelude::DispatcherBuilder,
};

#[derive(Default)]
pub struct LiveBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for LiveBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        TuiBundle::new().build(builder)?;
        builder.add(
            DetectPlayerActionSystem::default(),
            "detect_player_action",
            &[],
        );
        builder.add(LogDisplaySystem::default(), "log_display", &[]);
        builder.add(InventoryDisplaySystem::default(), "inventory_display", &[]);
        builder.add(
            ApplyBoardPositionSystem::default(),
            "apply_board_position",
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
        builder.add(
            ApplyWorldMapSystem::default(),
            "apply_worldmap",
            &["apply_attacks_2", "apply_movement_2"],
        );
        builder.add(StunSystem, "stun_system", &["apply_attacks_2"]);
        Ok(())
    }
}
