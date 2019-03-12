use super::{Parent, ParentHierarchy, TuiChannel, TuiEvent, Visible};
use crate::specs_ext::SpecsExt;
use amethyst::ecs::{prelude::*, SystemData as _};
use hibitset::BitSetLike;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct VisibleIfChildIs;

impl Component for VisibleIfChildIs {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct VisibilityRelationSystem {
    tui_reader: Option<ReaderId<TuiEvent>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    visible: WriteStorage<'s, Visible>,
    visible_if_child: ReadStorage<'s, VisibleIfChildIs>,
    tui_channel: Read<'s, TuiChannel>,
    parent: ReadStorage<'s, Parent>,
    parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
}

impl<'s> System<'s> for VisibilityRelationSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = BitSet::new();

        for event in data.tui_channel.read(self.tui_reader.as_mut().unwrap()) {
            match event {
                TuiEvent::Visible { entity, .. } => {
                    dirty.add(entity.id());
                }
                _ => {}
            }
        }

        if dirty.is_empty() {
            return;
        }

        for (parent, _) in (&data.parent, &dirty).join() {
            if data.visible_if_child.contains(parent.entity) {
                let vis = data
                    .parent_hierarchy
                    .children(parent.entity)
                    .iter()
                    .find(|x| data.visible.get(**x).map(|x| x.0).unwrap_or(true))
                    .is_some();
                data.visible.get_mut_or_default(parent.entity).0 = vis;
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.tui_reader = Some(res.get_mut::<TuiChannel>().unwrap().register_reader());
    }
}
