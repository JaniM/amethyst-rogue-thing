use amethyst::ecs::prelude::*;

use std::{cell::RefCell, rc::Rc};

pub use super::blink::BlinkSystem;
pub use super::{components::*, TuiChannel, TuiEvent};
pub use amethyst::core::transform::{Parent, ParentHierarchy};

use crate::specs_ext::SpecsExt;

use easycurses::*;

use hibitset::BitSetLike;

#[derive(Default)]
pub struct OldPosition(GlobalPosition);

impl Component for OldPosition {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default, Copy, Clone, PartialEq)]
pub struct ScreenSize {
    pub width: i32,
    pub height: i32,
}

pub struct TuiRenderSystem {
    easy: Rc<RefCell<EasyCurses>>,
    tui_reader: Option<ReaderId<TuiEvent>>,
    backplane: Vec<String>,
}

impl TuiRenderSystem {
    pub fn new(easy: Rc<RefCell<EasyCurses>>) -> Self {
        TuiRenderSystem {
            easy,
            tui_reader: None,
            backplane: Vec::new(),
        }
    }
}

#[derive(SystemData)]
pub struct TuiRenderSD<'s> {
    position: ReadStorage<'s, Position>,
    global_position: WriteStorage<'s, GlobalPosition>,
    text_block: ReadStorage<'s, TextBlock>,
    parent: ReadStorage<'s, Parent>,
    parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
    visible: ReadStorage<'s, Visible>,
    tui_channel: Read<'s, TuiChannel>,
    screen_size: Read<'s, ScreenSize>,
    zlevel: ReadStorage<'s, ZLevel>,
}

impl<'s> System<'s> for TuiRenderSystem {
    type SystemData = TuiRenderSD<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut easy = self.easy.borrow_mut();

        let mut dirty_local = BitSet::new();

        for event in data.tui_channel.read(self.tui_reader.as_mut().unwrap()) {
            use TuiEvent::*;
            let entity = match event {
                Position { entity, .. } => entity,
                TextBlock { entity, .. } => entity,
                Visible { entity, .. } => entity,
                HierarchyModified(entity) => entity,
                HierarchyRemoved(entity) => entity,
                ZLevel { entity, .. } => entity,
                GlobalPosition { .. } => {
                    continue;
                }
                ScreenSize { new, .. } => {
                    self.backplane = (0..new.height)
                        .map(|_| " ".repeat(new.width as usize))
                        .collect();
                    for (i, row) in self.backplane.iter().enumerate() {
                        easy.move_rc(i as i32, 0);
                        easy.print(row);
                    }
                    continue;
                }
            };
            dirty_local.add(entity.id());
        }

        if dirty_local.is_empty() {
            return;
        }

        let mut invisible = BitSet::new();

        let mut swap: Vec<Vec<(i32, char)>> = (0..data.screen_size.height)
            .map(|_| {
                " ".repeat(data.screen_size.width as usize)
                    .chars()
                    .map(|x| (-100, x))
                    .collect()
            })
            .collect();

        for entity in data.parent_hierarchy.all() {
            if data.visible.get(*entity) == Some(&Visible(false)) {
                invisible.add(entity.id());
                continue;
            }

            if let (Some(local), Some(text_block)) =
                (data.position.get(*entity), data.text_block.get(*entity))
            {
                let mut global = local.global();
                if let Some(parent) = data.parent.get(*entity) {
                    if invisible.contains(parent.entity.id()) {
                        invisible.add(entity.id());
                        continue;
                    }
                    let pos = if let Some(pg) = data.global_position.get(parent.entity) {
                        (pg.0 + local).global()
                    } else {
                        local.global()
                    };

                    *data.global_position.get_mut_or_default(*entity) = pos;
                    global = pos;
                }

                let zlevel = data.zlevel.get(*entity).map(|x| x.0).unwrap_or(0);

                for (i, row) in text_block
                    .rows
                    .iter()
                    .chain(["".to_owned()].iter().cycle())
                    .enumerate()
                    .take(text_block.height as usize)
                {
                    let y = i + global.0.y as usize;
                    let extra_width = i32::max(text_block.width - row.len() as i32, 0) as usize;
                    swap[y] = swap[y]
                        .iter()
                        .take(global.0.x as usize)
                        .cloned()
                        .chain(
                            row.chars()
                                .chain(" ".repeat(extra_width).chars())
                                .map(|x| (zlevel, x))
                                .zip(
                                    swap[y]
                                        .iter()
                                        .skip(global.0.x as usize)
                                        .take(text_block.width as usize),
                                )
                                .map(|(new, old)| if new.0 >= old.0 { new } else { *old }),
                        )
                        .chain(
                            swap[y]
                                .iter()
                                .skip((global.0.x + text_block.width) as usize)
                                .cloned(),
                        )
                        .take(swap[y].len())
                        .collect();
                }
            }
        }

        for y in 0..data.screen_size.height as usize {
            for (x, (old, new)) in self.backplane[y]
                .chars()
                .zip(swap[y].iter().map(|x| x.1))
                .enumerate()
            {
                if old != new {
                    easy.move_rc(y as i32, x as i32);
                    easy.print_char(new);
                }
            }
        }

        self.backplane = swap
            .into_iter()
            .map(|x| x.into_iter().map(|x| x.1).collect())
            .collect();

        easy.refresh();
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

        self.tui_reader = Some(res.get_mut::<TuiChannel>().unwrap().register_reader());
    }
}
