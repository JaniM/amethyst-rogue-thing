use amethyst::ecs::prelude::*;
use crossbeam_channel as channel;

use crate::{
    components::WorldPosition,
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
}

pub type MovementActions = MpscChannel<(Entity, Direction)>;
pub type AttackActions = MpscChannel<Attack>;

#[derive(Default, Debug, Clone)]
pub struct PlayerEntity(pub Option<Entity>);

#[derive(Default, Debug, Clone)]
pub struct WorldMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Option<Entity>>>,
}

impl WorldMap {
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> Self {
        WorldMap {
            width,
            height,
            tiles: (0..height)
                .into_iter()
                .map(|_| (0..width).into_iter().map(|_| None).collect())
                .collect(),
        }
    }

    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.tiles[y][x] = None;
            }
        }
    }

    pub fn read(&self, pos: &WorldPosition) -> Option<Entity> {
        if self.is_legal_pos(pos) {
            self.tiles[pos.y as usize][pos.x as usize]
        } else {
            None
        }
    }

    pub fn is_legal_pos(&self, pos: &WorldPosition) -> bool {
        pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32
    }
}

pub struct WorldPositionReader(pub ReaderId<ComponentEvent>);
