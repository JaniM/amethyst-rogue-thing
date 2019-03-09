use amethyst::ecs::{Component, DenseVecStorage, Entity, FlaggedStorage, NullStorage};

use crate::data::{Direction, Item};

pub use amethyst::core::Named;

#[derive(Default, Debug, Copy, Clone)]
pub struct PlayerControlledCharacter;

impl Component for PlayerControlledCharacter {
    type Storage = NullStorage<Self>;
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Character;

impl Component for Character {
    type Storage = NullStorage<Self>;
}

#[derive(Debug, Copy, Clone)]
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
}

impl Component for WorldPosition {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl WorldPosition {
    pub fn step_dir(&self, dir: Direction) -> WorldPosition {
        let mut wp = self.clone();
        match dir {
            Direction::Up => wp.y -= 1,
            Direction::Down => wp.y += 1,
            Direction::Left => wp.x -= 1,
            Direction::Right => wp.x += 1,
        }
        return wp;
    }
}

impl WorldPosition {
    pub fn new(x: i32, y: i32) -> Self {
        WorldPosition { x, y }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Team(pub u32);

impl Component for Team {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct AggressiveAI {
    pub aggressive_against: Vec<u32>,
}

impl Component for AggressiveAI {
    type Storage = DenseVecStorage<Self>;
}

impl AggressiveAI {
    pub fn new<'a, T>(against: T) -> Self
    where
        T: IntoIterator<Item = &'a u32>,
    {
        AggressiveAI {
            aggressive_against: against.into_iter().cloned().collect(),
        }
    }
}

#[derive(Default)]
pub struct Stunned {
    pub time: u32,
}

impl Component for Stunned {
    type Storage = DenseVecStorage<Self>;
}

impl Stunned {
    #[allow(dead_code)]
    pub fn new(time: u32) -> Self {
        Stunned { time }
    }
}

pub struct AggressionTarget {
    pub target: Entity,
}

impl Component for AggressionTarget {
    type Storage = DenseVecStorage<Self>;
}

impl AggressionTarget {
    pub fn new(target: Entity) -> Self {
        AggressionTarget { target }
    }
}

pub struct Health {
    pub health: i32,
}

impl Component for Health {
    type Storage = DenseVecStorage<Self>;
}

impl Health {
    pub fn new(health: i32) -> Self {
        Health { health }
    }
}

#[derive(Default)]
pub struct Dead;

impl Component for Dead {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct AnimateMovement {
    pub from: Option<WorldPosition>,
    pub to: Option<WorldPosition>,
    pub time: f32,
    pub used_time: f32,
}

#[derive(Default)]
pub struct LogDisplay;

impl Component for LogDisplay {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct InventoryDisplay;

impl Component for InventoryDisplay {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct BoardDisplay;

impl Component for BoardDisplay {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct Inventory {
    pub items: Vec<Item>,
}

impl Inventory {
    pub fn new(items: Vec<Item>) -> Self {
        Inventory { items }
    }
}

impl Component for Inventory {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
