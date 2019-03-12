use super::{Parent, ParentHierarchy, TuiChannel, TuiEvent, Visible};
use crate::specs_ext::{ComponentEventReader, SpecsExt};
use amethyst::ecs::{prelude::*, SystemData as _};
use hibitset::BitSetLike;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct VisibleIfChildIs;

impl Component for VisibleIfChildIs {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct VisibilityRelationSystem {
    visibility_reader: ComponentEventReader<Visible>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    visible: WriteStorage<'s, Visible>,
    visible_if_child: ReadStorage<'s, VisibleIfChildIs>,
    parent: ReadStorage<'s, Parent>,
    parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
}

impl<'s> System<'s> for VisibilityRelationSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = BitSet::new();
        self.visibility_reader
            .read_to_bitset(&data.visible, &mut dirty);

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

        self.visibility_reader.setup(&res);
    }
}
