use amethyst::ecs::prelude::*;
use std::ops::Add;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    pub fn global(self) -> GlobalPosition {
        GlobalPosition(self)
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<'a> Add<&'a Position> for Position {
    type Output = Position;

    fn add(self, rhs: &'a Position) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Component for Position {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GlobalPosition(pub Position);

impl GlobalPosition {
    #[allow(dead_code)]
    pub fn new(x: i32, y: i32) -> Self {
        GlobalPosition(Position::new(x, y))
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

#[allow(dead_code)]
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

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ZLevel(pub i32);

impl ZLevel {
    pub fn new(level: i32) -> Self {
        ZLevel(level)
    }
}

impl Component for ZLevel {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
