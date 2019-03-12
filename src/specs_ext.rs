use std::ops::Deref;

use amethyst::{
    ecs::{
        prelude::{ComponentEvent, ReaderId},
        storage,
        storage::GenericWriteStorage,
        BitSet, Component, Entity, Resources, Storage, SystemData as _, WriteStorage,
    },
    shrev,
};

pub trait SpecsExt<C> {
    fn get_mut_or_default(&mut self, ent: Entity) -> &mut C
    where
        Self: GenericWriteStorage,
        C: Default;
}

impl<'a, C> SpecsExt<C> for WriteStorage<'a, C>
where
    C: Component,
{
    fn get_mut_or_default(&mut self, ent: Entity) -> &mut C
    where
        C: Default,
    {
        if !self.get(ent).is_some() {
            self.insert(ent, Default::default()).unwrap();
        }
        self.get_mut(ent).unwrap()
    }
}

pub struct ComponentEventReader<C> {
    reader_id: Option<ReaderId<ComponentEvent>>,
    _component: std::marker::PhantomData<C>,
}

impl<C> Default for ComponentEventReader<C> {
    fn default() -> Self {
        ComponentEventReader {
            reader_id: None,
            _component: std::marker::PhantomData,
        }
    }
}

impl<C> ComponentEventReader<C>
where
    C: Component,
    C::Storage: storage::Tracked,
{
    pub fn setup(&mut self, res: &Resources) {
        self.reader_id = Some(WriteStorage::<C>::fetch(res).register_reader());
    }

    pub fn read<'a, D>(
        &mut self,
        source: &'a Storage<'a, C, D>,
    ) -> shrev::EventIterator<'a, ComponentEvent>
    where
        D: Deref<Target = storage::MaskedStorage<C>> + 'a,
    {
        source.channel().read(
            self.reader_id
                .as_mut()
                .expect("EventReader::setup wasn't called before use"),
        )
    }

    pub fn read_to_bitset<'a, D>(&mut self, source: &Storage<'a, C, D>, bitset: &mut BitSet)
    where
        D: Deref<Target = storage::MaskedStorage<C>> + 'a,
    {
        for event in source.channel().read(
            self.reader_id
                .as_mut()
                .expect("EventReader::setup wasn't called before use"),
        ) {
            match event {
                ComponentEvent::Modified(id)
                | ComponentEvent::Inserted(id)
                | ComponentEvent::Removed(id) => {
                    bitset.add(*id);
                }
            }
        }
    }
}
