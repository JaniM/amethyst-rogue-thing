use crate::components;
use amethyst::ecs::{self, prelude::*, shred::ResourceId};

pub trait EventSystem<'a> {
    type SystemData: ecs::SystemData<'a>;
    type Event: Sized;

    fn run(data: &Self::SystemData, event: &mut Self::Event);
}

impl<'a, E, A, B> EventSystem<'a> for (A, B)
where
    A: EventSystem<'a, Event = E>,
    B: EventSystem<'a, Event = E>,
{
    type SystemData = (A::SystemData, B::SystemData);
    type Event = E;

    fn run(data: &Self::SystemData, event: &mut Self::Event) {
        A::run(&data.0, event);
        B::run(&data.1, event);
    }
}

pub struct ReifiedEventSystem<'a, T>
where
    T: EventSystem<'a>,
{
    data: T::SystemData,
}

impl<'a, T> ReifiedEventSystem<'a, T>
where
    T: EventSystem<'a>,
{
    pub fn run(&self, event: &mut T::Event) {
        T::run(&self.data, event);
    }
}

impl<'a, T> SystemData<'a> for ReifiedEventSystem<'a, T>
where
    T: EventSystem<'a>,
{
    fn setup(res: &mut Resources) {
        #![allow(unused_variables)]

        T::SystemData::setup(&mut *res);
    }

    fn fetch(res: &'a Resources) -> Self {
        #![allow(unused_variables)]

        ReifiedEventSystem {
            data: T::SystemData::fetch(res),
        }
    }

    fn reads() -> Vec<ResourceId> {
        #![allow(unused_mut)]

        T::SystemData::reads()
    }

    fn writes() -> Vec<ResourceId> {
        #![allow(unused_mut)]

        T::SystemData::writes()
    }
}

pub struct WeaponDamage;

impl<'a> EventSystem<'a> for WeaponDamage {
    type SystemData = ReadStorage<'a, components::Inventory>;
    type Event = (Entity, i32);

    fn run(Inventory: &Self::SystemData, event: &mut Self::Event) {
        event.1 += Inventory.get(event.0).unwrap().items.len() as i32;
    }
}

pub struct Test;

impl<'a> EventSystem<'a> for Test {
    type SystemData = ();
    type Event = (Entity, i32);

    fn run(_: &Self::SystemData, event: &mut Self::Event) {
        event.1 += 10;
    }
}
