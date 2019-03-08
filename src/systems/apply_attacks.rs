use crate::{
    components::{Dead, Health, Named, Stunned},
    play::initialise_enemy,
    resources::{AttackActions, LogEvents},
    specs_ext::SpecsExt,
    tui::TextBlock,
};
use amethyst::ecs::prelude::*;

pub struct ApplyAttacksSystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    health: WriteStorage<'s, Health>,
    dead: WriteStorage<'s, Dead>,
    stun: WriteStorage<'s, Stunned>,
    attacks: Write<'s, AttackActions>,
    lazy: Read<'s, LazyUpdate>,
    log: Read<'s, LogEvents>,
    name: ReadStorage<'s, Named>,
}

impl<'s> System<'s> for ApplyAttacksSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        while let Ok(attack_event) = data.attacks.receiver().try_recv() {
            let target = attack_event.target;

            if data.dead.get(target).is_some() {
                data.log.send("Attacked a dead target");
            } else if let Some(health) = data.health.get_mut(target) {
                health.health -= 1;
                data.log.send(format!(
                    "{} (id {}) attacked {} (id {}): {} hp left",
                    data.name
                        .get(attack_event.attacker)
                        .map(|x| &*x.name)
                        .unwrap_or("Unknown"),
                    attack_event.attacker.id(),
                    data.name.get(target).map(|x| &*x.name).unwrap_or("Unknown"),
                    target.id(),
                    health.health
                ));

                data.stun.get_mut_or_default(target).time += 1;

                if health.health <= 0 {
                    data.dead.insert(target, Dead).ok();
                    data.log.send(format!(
                        "{} (id {}) died",
                        data.name.get(target).map(|x| &*x.name).unwrap_or("Unknown"),
                        target.id(),
                    ));
                    data.lazy.exec_mut(move |world| {
                        initialise_enemy(world);
                        if let Some(mat) = world.write_storage::<TextBlock>().get_mut(target) {
                            mat.rows[0] = "x".to_owned();
                        }
                    })
                }
            } else {
                data.log.send("Attacked an entity without Health");
            }
        }
    }
}
