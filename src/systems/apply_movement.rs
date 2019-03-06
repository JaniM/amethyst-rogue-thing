use crate::{
    components::WorldPosition,
    resources::{MovementActions, WorldMap, WorldPositionReader},
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
}

impl<'s> System<'s> for ApplyMovementSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let map = &mut data.map as &mut WorldMap;
        loop {
            let (entity, dir) = if let Ok(a) = data.movements.receiver().try_recv() {
                a
            } else {
                break;
            };
            let wp = data.worldpos.get_mut(entity).unwrap();
            let oldpos = *wp;
            *wp = wp.step_dir(dir);
            if !map.is_legal_pos(wp) {
                *wp = oldpos;
                println!("Movement out of bounds");
            } else if map.tiles[wp.y as usize][wp.x as usize].is_some() {
                *wp = oldpos;
                println!("Movement blocked");
            } else {
                map.tiles[oldpos.y as usize][oldpos.x as usize] = None;
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
            for (entity, wp) in (&data.entities, &data.worldpos).join() {
                map.tiles[wp.y as usize][wp.x as usize] = Some(entity);
            }
        } else {
            for (entity, wp, _) in (&data.entities, &data.worldpos, &dirty).join() {
                map.tiles[wp.y as usize][wp.x as usize] = Some(entity);
            }
        }
    }
}
