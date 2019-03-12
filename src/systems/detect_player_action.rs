use crate::{
    components::{
        Controlled, Inventory, InventoryDisplay, InventoryDisplayKind, PlayerControlledCharacter,
        WorldPosition,
    },
    data::{Direction, PlayerAction},
    resources::{LogEvents, PlayerActionResource, WorldMap},
    tui::Key,
};
use amethyst::{
    core::{shrev::EventChannel, timing::Time},
    ecs::{prelude::*, SystemData as _},
};

#[derive(Default)]
pub struct DetectPlayerActionSystem {
    reader: Option<ReaderId<Key>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    inputs: Read<'s, EventChannel<Key>>,
    action: Write<'s, PlayerActionResource>,
    time: Read<'s, Time>,
    log: Read<'s, LogEvents>,
    player: ReadStorage<'s, PlayerControlledCharacter>,
    inventory: WriteStorage<'s, Inventory>,
    inventory_display: WriteStorage<'s, InventoryDisplay>,
    position: ReadStorage<'s, WorldPosition>,
    controlled: WriteStorage<'s, Controlled>,
    world_map: Write<'s, WorldMap>,
    entities: Entities<'s>,
}

impl<'s> System<'s> for DetectPlayerActionSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        use Direction::*;

        let mut action = None;
        let mut remove_control = None;

        for (entity, display, _active) in (
            &data.entities,
            &mut data.inventory_display,
            &data.controlled,
        )
            .join()
        {
            for key in data.inputs.read(self.reader.as_mut().unwrap()) {
                match key {
                    Key::Character('\u{1b}') => action = Some(PlayerAction::Quit),
                    Key::Character('q') => {
                        remove_control = Some(entity);
                        display.cursor_pos = None;
                    }
                    Key::Character('w') => {
                        display.cursor_pos = Some(display.cursor_pos.map_or(0, |x| 0.max(x - 1)));
                    }
                    Key::Character('s') => {
                        display.cursor_pos = Some(display.cursor_pos.map_or(0, |x| 0.max(x + 1)));
                    }
                    Key::Character(' ') => {
                        if display.display_kind == InventoryDisplayKind::Ground {
                            for (position, inventory, _player) in
                                (&data.position, &mut data.inventory, &data.player).join()
                            {
                                if let Some(tile) = data.world_map.get_mut(position) {
                                    if tile.items.len() > display.cursor_pos.unwrap_or(0) as usize {
                                        let item = tile
                                            .items
                                            .remove(display.cursor_pos.unwrap_or(0) as usize);
                                        data.entities.delete(item.entity).ok();
                                        data.log
                                            .send(format!("Grabbed {}", item.item.description()));
                                        inventory.items.push(item.item);
                                        if tile.items.len() == 0 {
                                            remove_control = Some(entity);
                                            display.cursor_pos = None;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    x => {
                        data.log.send(format!("Unrecognized input: {:?}", x));
                    }
                }
            }
        }

        if let Some(entity) = remove_control {
            data.controlled.remove(entity);
        }

        for key in data.inputs.read(self.reader.as_mut().unwrap()) {
            match key {
                Key::Character('\u{1b}') => action = Some(PlayerAction::Quit),
                Key::Character('w') => action = Some(PlayerAction::Move(Up)),
                Key::Character('s') => action = Some(PlayerAction::Move(Down)),
                Key::Character('a') => action = Some(PlayerAction::Move(Left)),
                Key::Character('d') => action = Some(PlayerAction::Move(Right)),
                Key::Character('x') => action = Some(PlayerAction::Wait),
                Key::Character('g') => action = Some(PlayerAction::Grab),
                x => {
                    data.log.send(format!("Unrecognized input: {:?}", x));
                }
            }
        }

        if action == Some(PlayerAction::Grab) {
            for (entity, inventory) in (&data.entities, &mut data.inventory_display).join() {
                if inventory.display_kind == InventoryDisplayKind::Ground {
                    inventory.cursor_pos = Some(0);
                    data.controlled.insert(entity, Controlled).ok();
                }
            }
            action = None;
        }

        data.action.hold_delay -= data.time.delta_seconds();

        if action.is_some() && data.action.hold_delay <= 0.0 {
            data.action.action = action;
            data.action.hold_delay = 0.25;
        } else if action.is_none() {
            data.action.hold_delay = 0.0;
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        res.insert(EventChannel::<Key>::new());
        self.reader = Some(
            res.get_mut::<EventChannel<Key>>()
                .unwrap()
                .register_reader(),
        );
    }
}
