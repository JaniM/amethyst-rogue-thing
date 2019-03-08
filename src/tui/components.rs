use amethyst::ecs::prelude::*;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    pub fn to_global(&self) -> GlobalPosition {
        GlobalPosition::new(self.x, self.y)
    }
}

impl Component for Position {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GlobalPosition {
    pub x: i32,
    pub y: i32,
}

impl GlobalPosition {
    pub fn new(x: i32, y: i32) -> Self {
        GlobalPosition { x, y }
    }
}

impl Component for GlobalPosition {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TextBlock {
    pub rows: Vec<String>,
    pub width: i32,
    pub height: i32,
}

impl TextBlock {
    pub fn new<T, K>(rows: T, width: i32, height: i32) -> Self
    where
        T: IntoIterator<Item = K>,
        K: Into<String>,
    {
        TextBlock {
            rows: rows.into_iter().map(|x| x.into()).collect(),
            width,
            height,
        }
    }

    pub fn single_row<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        let string = text.into();
        let len = string.len();
        TextBlock::new(vec![string], len as i32, 1)
    }

    pub fn empty(width: i32, height: i32) -> Self {
        TextBlock::new(Vec::<String>::new(), width, height)
    }
}

impl Component for TextBlock {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[derive(Debug, Clone)]
pub struct Blink {
    pub interval: f32,
    pub elapsed: f32,
}

impl Blink {
    pub fn new(interval: f32) -> Self {
        Blink {
            interval,
            elapsed: 0.0,
        }
    }
}

impl Component for Blink {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Visible(pub bool);

impl Visible {
    pub fn new(visible: bool) -> Self {
        Visible(visible)
    }
}

impl Default for Visible {
    fn default() -> Self {
        Visible::new(true)
    }
}

impl Component for Visible {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
