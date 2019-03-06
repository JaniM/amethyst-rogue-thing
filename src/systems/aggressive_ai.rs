use crate::{
    components::{
        AggressionTarget, AggressiveAI, Dead, PlayerControlledCharacter, Stunned, Team,
        WorldPosition,
    },
    data::{Attack, Direction},
    resources::{AttackActions, MovementActions},
};
use amethyst::ecs::prelude::*;

pub struct AggressiveAISystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    worldpos: ReadStorage<'s, WorldPosition>,
    ai: ReadStorage<'s, AggressiveAI>,
    teams: ReadStorage<'s, Team>,
    movements: Read<'s, MovementActions>,
    attacks: Read<'s, AttackActions>,
    entities: Entities<'s>,
    player: ReadStorage<'s, PlayerControlledCharacter>,
    stunned: ReadStorage<'s, Stunned>,
    target: WriteStorage<'s, AggressionTarget>,
    dead: ReadStorage<'s, Dead>,
}

impl<'s> System<'s> for AggressiveAISystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (entity, wp, ai, (), (), ()) in (
            &data.entities,
            &data.worldpos,
            &data.ai,
            !&data.player,
            !&data.stunned,
            !&data.dead,
        )
            .join()
        {
            let target = if let Some(t) = data.target.get(entity) {
                Some(t.target)
            } else {
                let mut target = None;
                for (target_entity, _target_wp, target_team, ()) in
                    (&data.entities, &data.worldpos, &data.teams, !&data.dead).join()
                {
                    if ai.aggressive_against.contains(&target_team.0) {
                        target = Some(target_entity);
                        break;
                    }
                }
                if let Some(t) = target {
                    data.target
                        .insert(entity, AggressionTarget::new(t))
                        .expect("Adding AggressionTarget failed");
                }
                target
            };
            if let Some(target) = target {
                if data.dead.get(target).is_some() {
                    data.target
                        .remove(entity)
                        .expect("Removing AggressionTarget failed");
                    continue;
                }
                let target_wp = data
                    .worldpos
                    .get(target)
                    .expect("Aggression target has no WorldPosition");

                use Direction::*;
                let dx = (target_wp.x - wp.x).abs();
                let dy = (target_wp.y - wp.y).abs();
                let dir = if dx > dy && dx > 1 {
                    if target_wp.x < wp.x {
                        Some(Left)
                    } else {
                        Some(Right)
                    }
                } else if dy > 1 {
                    if target_wp.y < wp.y {
                        Some(Down)
                    } else {
                        Some(Up)
                    }
                } else {
                    None
                };

                if let Some(dir) = dir {
                    data.movements
                        .sender()
                        .send((entity, dir))
                        .expect("Send failed");
                } else {
                    data.attacks
                        .sender()
                        .send(Attack {
                            attacker: entity,
                            target,
                        })
                        .expect("Send failed");
                }
            }
        }
    }
}
