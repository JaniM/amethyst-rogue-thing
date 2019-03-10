use crate::{
    components::{PlayerControlledCharacter, WorldPosition},
    resources::{LogEvents, MovementActions, WorldMap},
};
use amethyst::ecs::prelude::*;

pub struct ApplyMovementSystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    worldpos: WriteStorage<'s, WorldPosition>,
    movements: Write<'s, MovementActions>,
    map: Write<'s, WorldMap>,
    log: Read<'s, LogEvents>,
    player: ReadStorage<'s, PlayerControlledCharacter>,
}

impl<'s> System<'s> for ApplyMovementSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let map = &mut data.map as &mut WorldMap;
        while let Ok((entity, dir)) = data.movements.receiver().try_recv() {
            let wp = data.worldpos.get_mut(entity).unwrap();
            let oldpos = *wp;
            *wp = wp.step_dir(dir);
            if !map.is_legal_pos(wp) {
                *wp = oldpos;
                data.log.send("Movement out of bounds");
            } else if map.read(wp).is_some() {
                *wp = oldpos;
                data.log.send("Movement blocked");
            } else {
                map.tiles[oldpos.y as usize][oldpos.x as usize].character = None;
            }

            if data.player.contains(entity) {
                if let Some(tile) = map.get(wp) {
                    for item in &tile.items {
                        data.log
                            .send(format!("You see a {}", item.item.description()));
                    }
                }
            }
        }
    }
}
