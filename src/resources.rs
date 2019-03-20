use amethyst::ecs::prelude::*;
use crossbeam_channel as channel;

use crate::{
    components::{Item, WorldPosition},
    data::{Attack, Direction, PlayerAction},
};

#[derive(Default, Debug, Clone)]
pub struct PlayerActionResource {
    pub action: Option<PlayerAction>,
    pub hold_delay: f32,
}

#[derive(Debug, Clone)]
pub struct MpscChannel<D> {
    sender: channel::Sender<D>,
    receiver: channel::Receiver<D>,
}

impl<D> Default for MpscChannel<D> {
    fn default() -> Self {
        MpscChannel::new()
    }
}

impl<D> MpscChannel<D> {
    pub fn new() -> Self {
        let (sender, receiver) = channel::unbounded();
        MpscChannel { sender, receiver }
    }

    pub fn receiver(&mut self) -> &channel::Receiver<D> {
        return &self.receiver;
    }

    pub fn sender(&self) -> &channel::Sender<D> {
        return &self.sender;
    }

    pub fn send<T>(&self, value: T)
    where
        T: Into<D>,
    {
        self.sender.send(value.into()).expect("Send failed");
    }
}

pub type MovementActions = MpscChannel<(Entity, Direction)>;
pub type AttackActions = MpscChannel<Attack>;

#[derive(Default, Debug, Clone)]
pub struct PlayerEntity(pub Option<Entity>);

#[derive(Debug, Clone)]
pub struct WorldItem {
    pub entity: Entity,
    pub item: Item,
}

#[derive(Default, Debug, Clone)]
pub struct WorldTile {
    pub character: Option<Entity>,
    pub items: Vec<WorldItem>,
}

#[derive(Default, Debug, Clone)]
pub struct WorldMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<WorldTile>>,
}

impl WorldMap {
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> Self {
        WorldMap {
            width,
            height,
            tiles: (0..height)
                .into_iter()
                .map(|_| {
                    (0..width)
                        .into_iter()
                        .map(|_| WorldTile::default())
                        .collect()
                })
                .collect(),
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.tiles[y][x] = WorldTile::default();
            }
        }
    }

    pub fn read(&self, pos: &WorldPosition) -> Option<Entity> {
        if self.is_legal_pos(pos) {
            self.tiles[pos.y as usize][pos.x as usize].character
        } else {
            None
        }
    }

    pub fn get(&self, pos: &WorldPosition) -> Option<&WorldTile> {
        if self.is_legal_pos(pos) {
            Some(&self.tiles[pos.y as usize][pos.x as usize])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, pos: &WorldPosition) -> Option<&mut WorldTile> {
        if self.is_legal_pos(pos) {
            Some(&mut self.tiles[pos.y as usize][pos.x as usize])
        } else {
            None
        }
    }

    pub fn is_legal_pos(&self, pos: &WorldPosition) -> bool {
        pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32
    }
}

#[derive(Default, Debug, Clone)]
pub struct EventLog {
    pub events: Vec<String>,
}

pub struct LogLine(pub String);

impl<T> From<T> for LogLine
where
    T: Into<String>,
{
    fn from(value: T) -> LogLine {
        LogLine(value.into())
    }
}

pub type LogEvents = MpscChannel<LogLine>;

#[derive(Default)]
pub struct Board(pub Option<Entity>);

#[derive(Default)]
pub struct TurnCounter(pub i32);
