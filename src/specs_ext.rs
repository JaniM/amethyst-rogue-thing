use amethyst::ecs::{storage::GenericWriteStorage, Component, Entity, WriteStorage};

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
