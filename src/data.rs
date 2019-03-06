use amethyst::ecs::Entity;

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
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Attack {
    pub attacker: Entity,
    pub target: Entity,
}
