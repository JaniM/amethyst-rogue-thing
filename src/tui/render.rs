use amethyst::{
    core::transform::{HierarchyEvent, Parent, ParentHierarchy},
    ecs::prelude::*,
};

use std::{cell::RefCell, rc::Rc};

pub use super::blink::BlinkSystem;
pub use super::components::*;
use crate::specs_ext::SpecsExt;

use easycurses::*;

#[derive(Default)]
pub struct OldPosition(GlobalPosition);

impl Component for OldPosition {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct ScreenSize {
    pub width: i32,
    pub height: i32,
}

pub struct TuiRenderSystem {
    easy: Rc<RefCell<EasyCurses>>,
    position_reader: Option<ReaderId<ComponentEvent>>,
    textblock_reader: Option<ReaderId<ComponentEvent>>,
    parent_reader: Option<ReaderId<HierarchyEvent>>,
    visible_reader: Option<ReaderId<ComponentEvent>>,
}

impl TuiRenderSystem {
    pub fn new(easy: Rc<RefCell<EasyCurses>>) -> Self {
        TuiRenderSystem {
            easy,
            position_reader: None,
            textblock_reader: None,
            parent_reader: None,
            visible_reader: None,
        }
    }
}

#[derive(SystemData)]
pub struct TuiRenderSD<'s> {
    position: ReadStorage<'s, Position>,
    old_position: WriteStorage<'s, OldPosition>,
    global_position: WriteStorage<'s, GlobalPosition>,
    text_block: ReadStorage<'s, TextBlock>,
    parent: ReadStorage<'s, Parent>,
    parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
    entities: Entities<'s>,
    visible: ReadStorage<'s, Visible>,
}

impl<'s> System<'s> for TuiRenderSystem {
    type SystemData = TuiRenderSD<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut out = self.easy.borrow_mut();

        let mut dirty_local = BitSet::new();
        let mut dirty_global = BitSet::new();

        for event in data
            .position
            .channel()
            .read(self.position_reader.as_mut().unwrap())
            .chain(
                data.text_block
                    .channel()
                    .read(self.textblock_reader.as_mut().unwrap()),
            )
            .chain(
                data.visible
                    .channel()
                    .read(self.visible_reader.as_mut().unwrap()),
            )
        {
            match event {
                ComponentEvent::Modified(id) | ComponentEvent::Inserted(id) => {
                    dirty_local.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    dirty_local.add(*id);
                }
            }
        }

        for event in data
            .parent_hierarchy
            .changed()
            .read(self.parent_reader.as_mut().unwrap())
        {
            match event {
                HierarchyEvent::Modified(entity) => {
                    dirty_local.add(entity.id());
                }
                HierarchyEvent::Removed(entity) => {
                    dirty_local.add(entity.id());
                }
            }
        }

        for (entity, local, _dirty) in (&data.entities, &data.position, &dirty_local).join() {
            let global = data.global_position.get_mut_or_default(entity);
            global.x = local.x;
            global.y = local.y;
            dirty_global.add(entity.id());
        }

        let mut invisible = BitSet::new();

        for entity in data.parent_hierarchy.all() {
            let self_dirty = dirty_local.contains(entity.id());
            if data.visible.get(*entity) == Some(&Visible(false)) {
                invisible.add(entity.id());
            }
            if let (Some(parent), Some(local)) =
                (data.parent.get(*entity), data.position.get(*entity))
            {
                if invisible.contains(parent.entity.id()) {
                    invisible.add(parent.entity.id());
                }
                let parent_dirty = dirty_global.contains(parent.entity.id());
                if parent_dirty || self_dirty {
                    let pos = if let Some(pg) = data.global_position.get(parent.entity) {
                        GlobalPosition::new(pg.x + local.x, pg.y + local.y)
                    } else {
                        local.to_global()
                    };

                    if let Some(global) = data.global_position.get_mut(*entity) {
                        dirty_global.add(entity.id());
                        dirty_global.add(parent.entity.id());
                        for entity in data.parent_hierarchy.children(parent.entity) {
                            dirty_global.add(entity.id());
                        }
                        *global = pos;
                    }
                }
            }
        }

        let mut rendered = BitSet::new();

        for (entity, _dirty) in (&data.entities, &dirty_global).join() {
            if invisible.contains(entity.id()) {
                rendered.add(entity.id());
            }
            if rendered.contains(entity.id()) {
                continue;
            }
            let mut next_parent = Some(entity);
            let mut parents = Vec::new();
            while let Some(p_entity) = next_parent {
                if !dirty_global.contains(p_entity.id()) {
                    break;
                }
                next_parent = data.parent_hierarchy.parent(p_entity);
                if rendered.contains(p_entity.id()) {
                    continue;
                }
                if let Some(block) = data.text_block.get(p_entity) {
                    rendered.add(p_entity.id());
                    let pos = data.global_position.get(p_entity).unwrap();
                    let old_pos = data.old_position.get_mut_or_default(p_entity);
                    parents.push(p_entity);
                    if &old_pos.0 != pos {
                        for i in 0..block.height {
                            out.move_rc(old_pos.0.y + i as i32, old_pos.0.x);
                            out.print(" ".repeat(block.width as usize));
                        }
                        old_pos.0 = pos.clone();
                    }
                    for i in 0..block.height {
                        out.move_rc(pos.y + i as i32, pos.x);
                        out.print(" ".repeat(block.width as usize));
                    }
                }
            }
            parents.reverse();
            let mut max_width = data.text_block.get(parents[0]).unwrap().width;
            let mut max_height = data.text_block.get(parents[0]).unwrap().height;
            for p_entity in parents {
                let block = data.text_block.get(p_entity).unwrap();
                let pos = data.global_position.get(p_entity).unwrap();
                for (i, row) in block.rows.iter().enumerate().take(max_height as usize) {
                    let short = row.chars().take(max_width as usize).collect::<String>();
                    out.move_rc(pos.y + i as i32, pos.x);
                    out.print(short);
                }
                max_width = block.width.min(max_width);
                max_height = block.height.min(max_height);
            }
        }
        out.refresh();
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        let mut easy = self.easy.borrow_mut();
        easy.set_cursor_visibility(CursorVisibility::Invisible);
        easy.set_echo(false);
        easy.set_keypad_enabled(true);
        easy.set_input_mode(InputMode::Character);
        easy.set_input_timeout(TimeoutMode::Immediate);
        easy.set_scrolling(true);
        easy.set_color_pair(ColorPair::new(Color::White, Color::Black));

        let (height, width) = easy.get_row_col_count();
        res.get_mut::<crate::resources::LogEvents>()
            .unwrap()
            .send(format!("({}, {})", height, width));

        res.insert(ScreenSize { width, height });

        self.position_reader = Some(WriteStorage::<Position>::fetch(&res).register_reader());
        self.textblock_reader = Some(WriteStorage::<TextBlock>::fetch(&res).register_reader());
        self.visible_reader = Some(WriteStorage::<Visible>::fetch(&res).register_reader());
        self.parent_reader = Some(res.fetch_mut::<ParentHierarchy>().track());
    }
}
