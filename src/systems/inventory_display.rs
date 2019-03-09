use crate::{
    components::{Inventory, InventoryDisplay, PlayerControlledCharacter},
    data::calculate_hash,
    tui::TextBlock,
};
use amethyst::ecs::prelude::*;

#[derive(Default)]
pub struct InventoryDisplaySystem {
    old_inventory_hash: u64,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    text_block: WriteStorage<'s, TextBlock>,
    inventory_display: ReadStorage<'s, InventoryDisplay>,
    inventory: ReadStorage<'s, Inventory>,
    player: ReadStorage<'s, PlayerControlledCharacter>,
}

impl<'s> System<'s> for InventoryDisplaySystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (inventory, _player) in (&data.inventory, &data.player).join() {
            let dirty = calculate_hash(&inventory.items) != self.old_inventory_hash;
            if dirty {
                for (block, _) in (&mut data.text_block, &data.inventory_display).join() {
                    block.rows = ["| Inventory", "| "]
                        .into_iter()
                        .map(|x| (*x).to_owned())
                        .chain(
                            inventory
                                .items
                                .iter()
                                .map(|item| item.description())
                                .chain(["".to_owned()].into_iter().cycle().cloned())
                                .map(|x| {
                                    ("| * ".to_owned() + &x)
                                        .chars()
                                        .chain([' '].into_iter().cycle().cloned())
                                        .take(block.width as usize - 1)
                                        .collect()
                                }),
                        )
                        .take(block.height as usize)
                        .collect();
                }
            }
        }
    }
}
