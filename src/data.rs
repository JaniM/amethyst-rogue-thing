use amethyst::ecs::prelude::*;
use std::{
    borrow::Cow,
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

#[derive(Default, Debug, Clone, PartialEq, Hash)]
pub struct ItemProperties {
    pub name: Cow<'static, str>,
    pub damage: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ItemPart {
    Name(Cow<'static, str>),
    Damage(i32),
}

impl ItemPart {
    pub fn collect_properties(&self, prop: &mut ItemProperties) {
        use ItemPart::*;
        match *self {
            Name(ref name) => {
                prop.name.to_mut().push_str(name);
            }
            Damage(dmg) => {
                prop.damage = Some(prop.damage.map_or(dmg, |x| x + dmg));
            }
        }
    }
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
