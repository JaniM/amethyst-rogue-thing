use crate::{
    components::{PlayerControlledCharacter, WorldPosition},
    resources::{LogEvents, MovementActions, WorldMap, WorldPositionReader},
    tui::Position,
};
use amethyst::ecs::prelude::*;

pub struct ApplyMovementSystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    worldpos: WriteStorage<'s, WorldPosition>,
    movements: Write<'s, MovementActions>,
    map: Write<'s, WorldMap>,
    entities: Entities<'s>,
    reader: WriteExpect<'s, WorldPositionReader>,
    screenpos: WriteStorage<'s, Position>,
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

        let mut dirty = BitSet::new();
        let mut redo = false;

        for event in data.worldpos.channel().read(&mut data.reader.0) {
            match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    dirty.add(*id);
                }
                ComponentEvent::Removed(_) => {
                    redo = true;
                }
            }
        }

        if redo {
            map.clear();
            for (entity, wp, pos) in (
                &data.entities,
                &data.worldpos,
                (&mut data.screenpos).maybe(),
            )
                .join()
            {
                map.tiles[wp.y as usize][wp.x as usize].character = Some(entity);
                if let Some(pos) = pos {
                    pos.x = wp.x;
                    pos.y = wp.y;
                }
            }
        } else {
            for (entity, wp, pos, _) in (
                &data.entities,
                &data.worldpos,
                (&mut data.screenpos).maybe(),
                &dirty,
            )
                .join()
            {
                map.tiles[wp.y as usize][wp.x as usize].character = Some(entity);
                if let Some(pos) = pos {
                    pos.x = wp.x;
                    pos.y = wp.y;
                }
            }
        }
    }
}
