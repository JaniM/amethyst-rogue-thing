use crate::{
    components::{
        Inventory, InventoryDisplay, InventoryDisplayKind, Item, PlayerControlledCharacter,
        WorldPosition,
    },
    data::calculate_hash,
    resources::WorldMap,
    specs_ext::SpecsExt,
    tui::{TextBlock, TuiChannel, TuiEvent, Visible},
};
use amethyst::ecs::{prelude::*, SystemData as _};
use std::borrow::Borrow;

#[derive(Default)]
pub struct InventoryDisplaySystem {
    old_inventory_hash: Option<u64>,
    old_ground_hash: Option<u64>,
    tui_reader: Option<ReaderId<TuiEvent>>,
    display_reader: Option<ReaderId<ComponentEvent>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    text_block: WriteStorage<'s, TextBlock>,
    inventory_display: ReadStorage<'s, InventoryDisplay>,
    inventory: ReadStorage<'s, Inventory>,
    player: ReadStorage<'s, PlayerControlledCharacter>,
    position: ReadStorage<'s, WorldPosition>,
    world_map: Read<'s, WorldMap>,
    tui_channel: Read<'s, TuiChannel>,
    entities: Entities<'s>,
    visible: WriteStorage<'s, Visible>,
}

impl<'s> System<'s> for InventoryDisplaySystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty_displays = false;
        let mut displays = BitSet::new();

        for (entity, _display) in (&data.entities, &data.inventory_display).join() {
            displays.add(entity.id());
        }

        for event in data
            .inventory_display
            .channel()
            .read(self.display_reader.as_mut().unwrap())
        {
            match event {
                ComponentEvent::Inserted(_)
                | ComponentEvent::Modified(_)
                | ComponentEvent::Removed(_) => {
                    dirty_displays = true;
                }
            }
        }

        for event in data.tui_channel.read(self.tui_reader.as_mut().unwrap()) {
            match event {
                TuiEvent::TextBlock {
                    entity,
                    new_size,
                    old_size,
                } => {
                    if new_size != old_size && !dirty_displays && displays.contains(entity.id()) {
                        dirty_displays = true;
                    }
                }
                _ => {}
            }
        }

        for (inventory, _player) in (&data.inventory, &data.player).join() {
            let hash = Some(calculate_hash(&inventory.items));
            let dirty = hash != self.old_inventory_hash || dirty_displays;
            self.old_inventory_hash = hash;
            if dirty {
                build_inventory(
                    &inventory.items,
                    &data.entities,
                    &mut data.text_block,
                    &data.inventory_display,
                    InventoryDisplayKind::Own,
                    &mut data.visible,
                    true,
                );
            }
        }
        for (position, _player) in (&data.position, &data.player).join() {
            if let Some(tile) = data.world_map.get(position) {
                let items = tile.items.iter().map(|x| &x.item).collect::<Vec<_>>();
                let hash = Some(calculate_hash(&items));
                let dirty = hash != self.old_ground_hash || dirty_displays;
                self.old_ground_hash = hash;
                if dirty {
                    build_inventory(
                        &items,
                        &data.entities,
                        &mut data.text_block,
                        &data.inventory_display,
                        InventoryDisplayKind::Ground,
                        &mut data.visible,
                        items.len() > 0,
                    );
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.tui_reader = Some(res.get_mut::<TuiChannel>().unwrap().register_reader());
        self.display_reader = Some(WriteStorage::<InventoryDisplay>::fetch(&res).register_reader());
    }
}

fn build_inventory<T>(
    items: &[T],
    entities: &Entities,
    text_block: &mut WriteStorage<TextBlock>,
    inventory_display: &ReadStorage<InventoryDisplay>,
    kind: InventoryDisplayKind,
    visible: &mut WriteStorage<Visible>,
    is_visible: bool,
) where
    T: Borrow<Item>,
{
    let title = match kind {
        InventoryDisplayKind::Own => "Inventory",
        InventoryDisplayKind::Ground => "Items on the ground",
    };
    for (entity, block, display) in (entities, text_block, inventory_display).join() {
        if display.display_kind != kind {
            continue;
        }
        visible.get_mut_or_default(entity).0 = is_visible || display.cursor_pos.is_some();
        block.rows = [title, ""]
            .into_iter()
            .map(|x| (*x).to_owned())
            .chain(
                items
                    .iter()
                    .map(|item| item.borrow().description())
                    .chain(["".to_owned()].into_iter().cycle().cloned())
                    .enumerate()
                    .map(|(i, x)| {
                        format!(
                            "*{} {}",
                            if Some(i as i32) == display.cursor_pos {
                                " >"
                            } else {
                                ""
                            },
                            x
                        )
                        .chars()
                        .chain([' '].into_iter().cycle().cloned())
                        .take(block.width as usize - 1)
                        .collect()
                    }),
            )
            .take(block.height as usize + 50)
            .collect();
    }
}
