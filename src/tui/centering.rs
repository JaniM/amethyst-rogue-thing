use super::{Parent, Position, TextBlock, TuiChannel, TuiEvent};
use crate::specs_ext::SpecsExt;
use amethyst::ecs::{prelude::*, SystemData as _};
use hibitset::BitSetLike;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Centered {
    pub horizontal: bool,
    pub vertical: bool,
}

impl Centered {
    pub fn new(horizontal: bool, vertical: bool) -> Self {
        Centered {
            horizontal,
            vertical,
        }
    }
}

impl Component for Centered {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct CenteringSystem {
    tui_reader: Option<ReaderId<TuiEvent>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    text_block: WriteStorage<'s, TextBlock>,
    position: WriteStorage<'s, Position>,
    centered: ReadStorage<'s, Centered>,
    tui_channel: Read<'s, TuiChannel>,
    entities: Entities<'s>,
    parent: ReadStorage<'s, Parent>,
}

impl<'s> System<'s> for CenteringSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = BitSet::new();

        for event in data.tui_channel.read(self.tui_reader.as_mut().unwrap()) {
            match event {
                TuiEvent::TextBlock { entity, .. } => {
                    dirty.add(entity.id());
                }
                _ => {}
            }
        }

        if dirty.is_empty() {
            return;
        }

        for (entity, centered, parent) in (&data.entities, &data.centered, &data.parent).join() {
            if dirty.contains(parent.entity.id()) {
                let p_block = data
                    .text_block
                    .get(parent.entity)
                    .expect("Centered entity's parent has no TextBlock");
                let block = data
                    .text_block
                    .get(entity)
                    .expect("Centered entity has no TextBlock");
                let position = data.position.get_mut_or_default(entity);

                if centered.horizontal {
                    position.x = p_block.width / 2 - block.width / 2;
                }
                if centered.vertical {
                    position.y = p_block.height / 2 - block.height / 2;
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.tui_reader = Some(res.get_mut::<TuiChannel>().unwrap().register_reader());
    }
}
