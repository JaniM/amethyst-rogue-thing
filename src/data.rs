use amethyst::ecs::prelude::*;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlayerAction {
    Move(Direction),
    Wait,
    Grab,
    Quit,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Attack {
    pub attacker: Entity,
    pub target: Entity,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Weapon {
    pub name: String,
    pub damage: i32,
}

impl Weapon {
    pub fn description(&self) -> String {
        format!("{} (ATK {})", self.name, self.damage)
    }
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
