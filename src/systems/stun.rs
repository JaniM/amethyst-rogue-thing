use crate::components::Stunned;
use amethyst::ecs::prelude::*;

pub struct StunSystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    stunned: WriteStorage<'s, Stunned>,
    entities: Entities<'s>,
}

impl<'s> System<'s> for StunSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut cleared = Vec::new();
        for (entity, stun) in (&data.entities, &mut data.stunned).join() {
            stun.time -= 1;
            if stun.time == 0 {
                cleared.push(entity);
            }
        }
        for entity in cleared {
            data.stunned.remove(entity);
        }
    }
}
