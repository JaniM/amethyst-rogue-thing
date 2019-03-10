use crate::{
    components::{Character, Item, WorldPosition},
    resources::{WorldItem, WorldMap},
};
use amethyst::ecs::{prelude::*, SystemData as _};

pub struct OldWorldPosition(pub WorldPosition);

impl Component for OldWorldPosition {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct ApplyWorldMapSystem {
    worldpos_reader: Option<ReaderId<ComponentEvent>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    worldpos: WriteStorage<'s, WorldPosition>,
    old_worldpos: WriteStorage<'s, OldWorldPosition>,
    map: Write<'s, WorldMap>,
    entities: Entities<'s>,
    character: ReadStorage<'s, Character>,
    item: ReadStorage<'s, Item>,
}

impl<'s> System<'s> for ApplyWorldMapSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let map = &mut data.map as &mut WorldMap;

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
            if let Some(OldWorldPosition(wp)) = data.old_worldpos.get(entity) {
                let tile = map.get_mut(wp).expect("Entity has illegal WorldPosition");
                if data.character.contains(entity) {
                    tile.character = None;
                } else if data.item.contains(entity) {
                    tile.items.retain(|x| x.entity == entity);
                }
            }
            if let Some(wp) = wp {
                let tile = map.get_mut(wp).expect("Entity has illegal WorldPosition");
                if data.character.contains(entity) {
                    tile.character = Some(entity);
                } else if let Some(item) = data.item.get(entity) {
                    tile.items.push(WorldItem {
                        entity,
                        item: item.clone(),
                    });
                }
                data.old_worldpos.insert(entity, OldWorldPosition(*wp)).ok();
            } else {
                data.old_worldpos.remove(entity);
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.worldpos_reader = Some(WriteStorage::<WorldPosition>::fetch(&res).register_reader());
    }
}
