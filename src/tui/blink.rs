use amethyst::{core::timing::Time, ecs::prelude::*};

pub use super::components::*;
use crate::specs_ext::SpecsExt;

#[derive(Default)]
pub struct BlinkSystem;

impl BlinkSystem {
    pub fn new() -> Self {
        BlinkSystem::default()
    }
}

#[derive(SystemData)]
pub struct BlinkSD<'s> {
    visible: WriteStorage<'s, Visible>,
    blink: WriteStorage<'s, Blink>,
    time: Read<'s, Time>,
    entities: Entities<'s>,
}

impl<'s> System<'s> for BlinkSystem {
    type SystemData = BlinkSD<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let delta = data.time.delta_seconds();
        for (entity, blink) in (&data.entities, &mut data.blink).join() {
            blink.elapsed += delta;
            if blink.elapsed > blink.interval {
                blink.elapsed -= blink.interval;
                let visible = data.visible.get_mut_or_default(entity);
                visible.0 = !visible.0;
            }
        }
    }
}
