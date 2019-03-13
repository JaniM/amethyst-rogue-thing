use super::{ParentHierarchy, Position, TextBlock};
use crate::specs_ext::{ComponentEventReader, SpecsExt};
use amethyst::ecs::{prelude::*, SystemData as _};
use hibitset::BitSetLike;

/// Border is the actual visual border.
/// Its *child* is what should be drawn with borders.
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Border {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
    pub cross_top_left: bool,
    pub cross_top_right: bool,
    pub cross_bottom_left: bool,
    pub cross_bottom_right: bool,
}

#[allow(dead_code)]
impl Border {
    pub fn new() -> Self {
        Border::default()
    }

    pub fn top(self) -> Self {
        Self { top: true, ..self }
    }
    pub fn bottom(self) -> Self {
        Self {
            bottom: true,
            ..self
        }
    }
    pub fn left(self) -> Self {
        Self { left: true, ..self }
    }
    pub fn right(self) -> Self {
        Self {
            right: true,
            ..self
        }
    }

    pub fn sides(self, left: bool, right: bool, top: bool, bottom: bool) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
            ..self
        }
    }

    pub fn crosses(
        self,
        cross_top_left: bool,
        cross_top_right: bool,
        cross_bottom_left: bool,
        cross_bottom_right: bool,
    ) -> Self {
        Self {
            cross_top_left,
            cross_top_right,
            cross_bottom_left,
            cross_bottom_right,
            ..self
        }
    }
}

impl Component for Border {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct BorderSystem {
    textblock_reader: ComponentEventReader<TextBlock>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    text_block: WriteStorage<'s, TextBlock>,
    position: WriteStorage<'s, Position>,
    border: ReadStorage<'s, Border>,
    entities: Entities<'s>,
    parent_hierarchy: ReadExpect<'s, ParentHierarchy>,
}

impl<'s> System<'s> for BorderSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = BitSet::new();

        self.textblock_reader
            .read_to_bitset(&data.text_block, &mut dirty);

        if dirty.is_empty() {
            return;
        }

        for (entity, border, _) in (&data.entities, &data.border, &dirty).join() {
            let (width, height) = {
                let block = data.text_block.get_mut_or_default(entity);
                (block.width, block.height)
            };

            for child in data.parent_hierarchy.children(entity) {
                let block = data.text_block.get_mut_or_default(*child);
                let position = data
                    .position
                    .get(*child)
                    .expect("Border child has no Posirion");

                block.width = width - position.x - if border.right { 1 } else { 0 };
                block.height = height - position.y - if border.bottom { 1 } else { 0 };
            }

            let block = data
                .text_block
                .get_mut(entity)
                .expect("Border entity has no TextBlock");

            let mut rows: Vec<Vec<char>> = (0..block.height)
                .map(|_| " ".repeat(block.width as usize).chars().collect())
                .collect();

            if border.left {
                for y in 0..block.height as usize {
                    rows[y][0] = '|';
                }
            }
            if border.right {
                for y in 0..block.height as usize {
                    rows[y][block.width as usize - 1] = '|';
                }
            }

            if border.top {
                for x in 0..block.width as usize {
                    rows[0][x] = '-';
                }
                if border.left {
                    rows[0][0] = '+';
                }
                if border.right {
                    rows[0][block.width as usize - 1] = '+';
                }
            }
            if border.bottom {
                for x in 0..block.width as usize {
                    rows[block.height as usize - 1][x] = '-';
                }
                if border.left {
                    rows[block.height as usize - 1][0] = '+';
                }
                if border.right {
                    rows[block.height as usize - 1][block.width as usize - 1] = '+';
                }
            }

            block.rows = rows.into_iter().map(|x| x.iter().collect()).collect();
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.textblock_reader.setup(&res);
    }
}
