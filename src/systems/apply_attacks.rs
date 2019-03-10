use crate::{
    components::{Dead, Health, Inventory, Named, Stunned, WorldPosition},
    play::initialise_item,
    resources::{AttackActions, Board, LogEvents, WorldMap},
    specs_ext::SpecsExt,
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
    world_map: Write<'s, WorldMap>,
    inventory: ReadStorage<'s, Inventory>,
    position: ReadStorage<'s, WorldPosition>,
    board: Read<'s, Board>,
    entities: Entities<'s>,
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
                    let position = data.position.get(target).unwrap();
                    let tile = data.world_map.get_mut(position).unwrap();
                    let mut itemc = 0;
                    tile.character = None;
                    if let Some(inventory) = data.inventory.get(target) {
                        let (entities, lazy, board) = (&data.entities, &data.lazy, &data.board);
                        inventory.items.iter().for_each(|item| {
                            initialise_item(
                                lazy.create_entity(entities),
                                board.0.unwrap(),
                                *position,
                                item.clone(),
                            )
                            .build();
                        });
                        itemc = inventory.items.len();
                    }
                    data.entities.delete(target).ok();
                    data.log.send(format!(
                        "{} (id {}) died{}",
                        data.name.get(target).map(|x| &*x.name).unwrap_or("Unknown"),
                        target.id(),
                        if itemc == 0 {
                            "".to_owned()
                        } else if itemc == 1 {
                            " and dropped 1 item".to_owned()
                        } else {
                            format!(" and dropped {} items", itemc)
                        }
                    ));
                    data.lazy.exec_mut(move |world| {
                        crate::play::initialise_enemy(world);
                    })
                }
            } else {
                data.log.send("Attacked an entity without Health");
            }
        }
    }
}
