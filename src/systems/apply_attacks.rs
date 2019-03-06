use crate::{
    components::{Dead, Health, Stunned},
    graphic::create_colour_material_static,
    play::initialise_enemy,
    resources::AttackActions,
};
use amethyst::{ecs::prelude::*, renderer::Material};

pub struct ApplyAttacksSystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    health: WriteStorage<'s, Health>,
    dead: WriteStorage<'s, Dead>,
    stun: WriteStorage<'s, Stunned>,
    attacks: Write<'s, AttackActions>,
    lazy: Read<'s, LazyUpdate>,
}

impl<'s> System<'s> for ApplyAttacksSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        loop {
            let attack_event = if let Ok(a) = data.attacks.receiver().try_recv() {
                a
            } else {
                break;
            };

            let target = attack_event.target;

            if data.dead.get(target).is_some() {
                println!("Attacked a dead target");
            } else if let Some(health) = data.health.get_mut(target) {
                health.health -= 1;
                println!("Dealt damage to {:?}: {} hp left", target, health.health);

                if let Some(stun) = data.stun.get_mut(target) {
                    stun.time += 1;
                } else {
                    data.stun
                        .insert(target, Stunned::new(1))
                        .expect("Adding Stun failed");
                }

                if health.health <= 0 {
                    data.dead.insert(target, Dead).ok();
                    data.lazy.exec_mut(move |world| {
                        initialise_enemy(world);
                        let material = create_colour_material_static(world, [1.0, 1.0, 0.0, 1.0]);
                        if let Some(mat) = world.write_storage::<Material>().get_mut(target) {
                            *mat = material
                        }
                    })
                }
            } else {
                println!("Attacked an entity without Health");
            }
        }
    }
}
