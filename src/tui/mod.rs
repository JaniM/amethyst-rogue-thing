
use amethyst::{prelude::*, ecs::prelude::*};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Position {
    pub x: u32;
    pub y: u32
}

impl Component for Position {
    type Storage = DenseVecStorage<Self>;
}

pub struct TextBlock {
    pub rows: Vec<String>;
}

impl Component for TextBlock {
    type Storage = DenseVecStorage<Self>;
}