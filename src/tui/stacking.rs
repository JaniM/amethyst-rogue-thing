use super::{Parent, ParentHierarchy, Position, ScreenSize, TextBlock, TuiChannel, TuiEvent};
use crate::specs_ext::SpecsExt;
use amethyst::ecs::{prelude::*, SystemData as _};
use hibitset::BitSetLike;

use crate::resources::LogEvents;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StackDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StackingContext {
    pub direction: StackDirection,
}

#[allow(dead_code)]
impl StackingContext {
    pub fn horizontal() -> Self {
        StackingContext {
            direction: StackDirection::Horizontal,
        }
    }

    pub fn vertical() -> Self {
        StackingContext {
            direction: StackDirection::Vertical,
        }
    }
}

impl Component for StackingContext {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StackingRule {
    pub max_width: Option<i32>,
    pub min_width: Option<i32>,
    pub max_height: Option<i32>,
    pub min_height: Option<i32>,
    pub flex: u32,
}

#[allow(dead_code)]
impl StackingRule {
    pub fn new() -> Self {
        StackingRule {
            max_width: None,
            min_width: None,
            max_height: None,
            min_height: None,
            flex: 1,
        }
    }
    pub fn max_width(mut self, value: i32) -> Self {
        self.max_width = Some(value);
        self
    }
    pub fn min_width(mut self, value: i32) -> Self {
        self.min_width = Some(value);
        self
    }
    pub fn max_height(mut self, value: i32) -> Self {
        self.max_height = Some(value);
        self
    }
    pub fn min_height(mut self, value: i32) -> Self {
        self.min_height = Some(value);
        self
    }
    pub fn flex(mut self, value: u32) -> Self {
        self.flex = value;
        self
    }
}

impl Component for StackingRule {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct StackingSystem {
    tui_reader: Option<ReaderId<TuiEvent>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    text_block: WriteStorage<'s, TextBlock>,
    screen_size: Read<'s, ScreenSize>,
    position: WriteStorage<'s, Position>,
    stacking_context: ReadStorage<'s, StackingContext>,
    stacking_rule: ReadStorage<'s, StackingRule>,
    tui_channel: Read<'s, TuiChannel>,
    entities: Entities<'s>,
    parent: ReadStorage<'s, Parent>,
    parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
    log: Read<'s, LogEvents>,
}

impl<'s> System<'s> for StackingSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty_contexts = BitSet::new();

        for event in data.tui_channel.read(self.tui_reader.as_mut().unwrap()) {
            match event {
                TuiEvent::ScreenSize { .. } => {
                    for (entity, _stc) in (&data.entities, &data.stacking_context).join() {
                        if !data.parent.contains(entity) {
                            dirty_contexts.add(entity.id());
                        }
                    }
                }
                _ => {}
            }
        }

        if dirty_contexts.is_empty() {
            return;
        }

        data.log.send("Stacking run");

        #[derive(Debug)]
        struct Resolved {
            entity: Entity,
            rule: StackingRule,
            frozen: bool,
        }

        for entity in (&data.entities, &data.stacking_context, !&data.parent)
            .join()
            .map(|x| x.0)
            .chain(data.parent_hierarchy.all().iter().cloned())
        {
            if !dirty_contexts.contains(entity.id()) {
                continue;
            }
            data.log.send(format!("Stacking entity: {:?}", entity));
            if !data.parent.contains(entity) {
                let block = data.text_block.get_mut_or_default(entity);
                block.width = data.screen_size.width;
                block.height = data.screen_size.height;
            }

            let context = data.stacking_context.get(entity).unwrap();

            let mut children = data
                .parent_hierarchy
                .children(entity)
                .iter()
                .filter(|x| data.stacking_rule.contains(**x))
                .cloned()
                .map(|x| Resolved {
                    entity: x,
                    rule: *data.stacking_rule.get(x).unwrap(),
                    frozen: false,
                })
                .collect::<Vec<_>>();

            data.log.send(format!("Stacking children: {:?}", children));

            let (total_width, total_height) = {
                let block = data
                    .text_block
                    .get(entity)
                    .expect("StackingContext doesn't have TextBlock");
                (block.width, block.height)
            };

            let mut used_size: i32 = 0;

            loop {
                let total_flex: u32 = children
                    .iter()
                    .filter(|x| !x.frozen)
                    .map(|x| x.rule.flex)
                    .sum();

                let mut pos: i32 = 0;
                let mut stack_changed = false;

                if context.direction == StackDirection::Horizontal {
                    let one_flex: f32 =
                        ((total_width - used_size) as f32 / total_flex as f32).floor();

                    for Resolved {
                        entity,
                        rule,
                        frozen,
                    } in &mut children
                    {
                        let width: i32 = (rule.flex as f32 * one_flex).floor() as i32;
                        let block = data.text_block.get_mut_or_default(*entity);
                        if !*frozen {
                            if let Some(min) = rule.min_width {
                                if width < min {
                                    block.width = min;
                                    used_size += min;
                                    *frozen = true;
                                    stack_changed = true;
                                }
                            }
                            if let Some(max) = rule.max_width {
                                if width > max {
                                    block.width = max;
                                    used_size += max;
                                    *frozen = true;
                                    stack_changed = true;
                                }
                            }
                        }
                        if !*frozen {
                            block.width = width;
                        }
                        block.height = total_height;
                        let position = data.position.get_mut_or_default(*entity);
                        position.x = pos;
                        position.y = 0;
                        pos += block.width;
                    }
                } else {
                    let one_flex: f32 =
                        ((total_height - used_size) as f32 / total_flex as f32).floor();

                    for Resolved {
                        entity,
                        rule,
                        frozen,
                    } in &mut children
                    {
                        let height: i32 = (rule.flex as f32 * one_flex).floor() as i32;
                        let block = data.text_block.get_mut_or_default(*entity);
                        if !*frozen {
                            if let Some(min) = rule.min_height {
                                if height < min {
                                    block.height = min;
                                    used_size += min;
                                    *frozen = true;
                                    stack_changed = true;
                                }
                            }
                            if let Some(max) = rule.max_height {
                                if height > max {
                                    block.height = max;
                                    used_size += max;
                                    *frozen = true;
                                    stack_changed = true;
                                }
                            }
                        }
                        if !*frozen {
                            block.height = height;
                        }
                        block.width = total_width;
                        let position = data.position.get_mut_or_default(*entity);
                        position.x = 0;
                        position.y = pos;
                        pos += block.height;
                    }
                }

                if !stack_changed {
                    break;
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.tui_reader = Some(res.get_mut::<TuiChannel>().unwrap().register_reader());
    }
}
