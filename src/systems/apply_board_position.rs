use crate::{components::WorldPosition, tui::Position};
use amethyst::ecs::{prelude::*, SystemData as _};

#[derive(Default)]
pub struct ApplyBoardPositionSystem {
    worldpos_reader: Option<ReaderId<ComponentEvent>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    worldpos: WriteStorage<'s, WorldPosition>,
    pos: WriteStorage<'s, Position>,
    entities: Entities<'s>,
}

impl<'s> System<'s> for ApplyBoardPositionSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = BitSet::new();

        for event in data
            .worldpos
            .channel()
            .read(self.worldpos_reader.as_mut().unwrap())
        {
            match event {
                ComponentEvent::Inserted(id)
                | ComponentEvent::Modified(id)
                | ComponentEvent::Removed(id) => {
                    dirty.add(*id);
                }
            }
        }

        for (entity, wp, _) in (&data.entities, data.worldpos.maybe(), &dirty).join() {
            if let Some(wp) = wp {
                data.pos.insert(entity, Position::new(wp.x, wp.y)).ok();
            } else {
                data.pos.remove(entity);
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.worldpos_reader = Some(WriteStorage::<WorldPosition>::fetch(&res).register_reader());
    }
}
