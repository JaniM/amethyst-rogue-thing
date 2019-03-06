use crate::{
    components::{Dead, PlayerControlledCharacter, Stunned, WorldPosition},
    data::{Attack, PlayerAction},
    resources::{AttackActions, MovementActions, PlayerActionResource, WorldMap},
};
use amethyst::ecs::prelude::*;

pub struct PlayerMovementSystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    control: ReadStorage<'s, PlayerControlledCharacter>,
    worldpos: ReadStorage<'s, WorldPosition>,
    action: Read<'s, PlayerActionResource>,
    movements: Read<'s, MovementActions>,
    entities: Entities<'s>,
    map: Read<'s, WorldMap>,
    attacks: Read<'s, AttackActions>,
    dead: ReadStorage<'s, Dead>,
    stun: ReadStorage<'s, Stunned>,
}

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, data: Self::SystemData) {
        match data.action.action {
            Some(PlayerAction::Move(dir)) => {
                for (entity, wp, _, (), ()) in (
                    &data.entities,
                    &data.worldpos,
                    &data.control,
                    !&data.dead,
                    !&data.stun,
                )
                    .join()
                {
                    if let Some(target) = data.map.read(&wp.step_dir(dir)) {
                        data.attacks
                            .sender()
                            .send(Attack {
                                attacker: entity,
                                target: target,
                            })
                            .expect("Send failed");
                    } else {
                        data.movements
                            .sender()
                            .send((entity, dir))
                            .expect("Send failed");
                    }
                }
            }
            Some(PlayerAction::Wait) => {}
            Some(PlayerAction::Quit) => {}
            None => {}
        }
    }
}
