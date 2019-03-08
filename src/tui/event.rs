use amethyst::{
    core::{
        shrev::EventChannel,
        transform::{HierarchyEvent, ParentHierarchy},
    },
    ecs::prelude::*,
};

use crate::specs_ext::SpecsExt;

pub use super::blink::BlinkSystem;
pub use super::{components::*, ScreenSize};

pub enum TuiEvent {
    Position {
        entity: Entity,
        new: Option<Position>,
        old: Option<Position>,
    },
    GlobalPosition {
        entity: Entity,
        new: Option<Position>,
        old: Option<Position>,
    },
    TextBlock {
        entity: Entity,
        new_size: Option<Position>,
        old_size: Option<Position>,
    },
    Visible {
        entity: Entity,
        new: Option<bool>,
        old: Option<bool>,
    },
    HierarchyModified(Entity),
    HierarchyRemoved(Entity),
    ScreenSize {
        new: ScreenSize,
        old: ScreenSize,
    },
}

pub type TuiChannel = EventChannel<TuiEvent>;

#[derive(Default)]
pub struct TuiStatus {
    position: Option<Position>,
    global_position: Option<Position>,
    text_block: Option<Position>,
    visible: Option<bool>,
}

impl Component for TuiStatus {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct TuiEventSystem {
    position_reader: Option<ReaderId<ComponentEvent>>,
    global_position_reader: Option<ReaderId<ComponentEvent>>,
    textblock_reader: Option<ReaderId<ComponentEvent>>,
    parent_reader: Option<ReaderId<HierarchyEvent>>,
    visible_reader: Option<ReaderId<ComponentEvent>>,

    screen_size: ScreenSize,
}

impl TuiEventSystem {
    pub fn new() -> Self {
        TuiEventSystem::default()
    }
}

#[derive(SystemData)]
pub struct TuiEventSD<'s> {
    position: ReadStorage<'s, Position>,
    global_position: WriteStorage<'s, GlobalPosition>,
    text_block: ReadStorage<'s, TextBlock>,
    parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
    entities: Entities<'s>,
    visible: ReadStorage<'s, Visible>,
    tui_status: WriteStorage<'s, TuiStatus>,
    events: Write<'s, TuiChannel>,
    screen_size: Read<'s, ScreenSize>,
}

impl<'s> System<'s> for TuiEventSystem {
    type SystemData = TuiEventSD<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = BitSet::new();

        read_events_to_bitset(
            data.position.channel(),
            &mut self.position_reader,
            &mut dirty,
        );

        for (local, entity, _id) in (data.position.maybe(), &data.entities, &dirty).join() {
            let tui_status = data.tui_status.get_mut_or_default(entity);
            let local = local.cloned();
            if local != tui_status.position {
                data.events.single_write(TuiEvent::Position {
                    entity,
                    new: local,
                    old: tui_status.position,
                });
                tui_status.position = local;
            }
        }

        dirty.clear();
        read_events_to_bitset(
            data.global_position.channel(),
            &mut self.global_position_reader,
            &mut dirty,
        );

        for (global, entity, _id) in (data.global_position.maybe(), &data.entities, &dirty).join() {
            let tui_status = data.tui_status.get_mut_or_default(entity);
            let global = global.cloned().map(|x| x.0);
            if global != tui_status.global_position {
                data.events.single_write(TuiEvent::GlobalPosition {
                    entity,
                    new: global,
                    old: tui_status.global_position,
                });
                tui_status.global_position = global;
            }
        }

        dirty.clear();
        read_events_to_bitset(
            data.text_block.channel(),
            &mut self.textblock_reader,
            &mut dirty,
        );

        for (text_block, entity, _id) in (data.text_block.maybe(), &data.entities, &dirty).join() {
            let tui_status = data.tui_status.get_mut_or_default(entity);
            let local = text_block.map(|x| Position::new(x.width, x.height));
            data.events.single_write(TuiEvent::TextBlock {
                entity,
                new_size: local,
                old_size: tui_status.text_block,
            });
            tui_status.text_block = local;
        }

        dirty.clear();
        read_events_to_bitset(data.visible.channel(), &mut self.visible_reader, &mut dirty);

        for (visible, entity, _id) in (data.visible.maybe(), &data.entities, &dirty).join() {
            let tui_status = data.tui_status.get_mut_or_default(entity);
            let visible = visible.map(|x| x.0);
            if visible != tui_status.visible {
                data.events.single_write(TuiEvent::Visible {
                    entity,
                    new: visible,
                    old: tui_status.visible,
                });
                tui_status.visible = visible;
            }
        }

        for event in data
            .parent_hierarchy
            .changed()
            .read(self.parent_reader.as_mut().unwrap())
        {
            data.events.single_write(match event {
                HierarchyEvent::Modified(entity) => TuiEvent::HierarchyModified(*entity),
                HierarchyEvent::Removed(entity) => TuiEvent::HierarchyRemoved(*entity),
            });
        }

        let screen_size = *data.screen_size;
        if screen_size != self.screen_size {
            data.events.single_write(TuiEvent::ScreenSize {
                new: screen_size,
                old: self.screen_size,
            });
            self.screen_size = screen_size;
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.position_reader = Some(WriteStorage::<Position>::fetch(&res).register_reader());
        self.global_position_reader =
            Some(WriteStorage::<GlobalPosition>::fetch(&res).register_reader());
        self.textblock_reader = Some(WriteStorage::<TextBlock>::fetch(&res).register_reader());
        self.visible_reader = Some(WriteStorage::<Visible>::fetch(&res).register_reader());
        self.parent_reader = Some(res.fetch_mut::<ParentHierarchy>().track());
    }
}

fn read_events_to_bitset(
    channel: &EventChannel<ComponentEvent>,
    reader: &mut Option<ReaderId<ComponentEvent>>,
    bitset: &mut BitSet,
) {
    for event in channel.read(reader.as_mut().unwrap()) {
        match event {
            ComponentEvent::Modified(id)
            | ComponentEvent::Inserted(id)
            | ComponentEvent::Removed(id) => {
                bitset.add(*id);
            }
        }
    }
}
