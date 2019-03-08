use amethyst::{core::shrev::EventChannel, ecs::prelude::*};

use super::render::ScreenSize;
use easycurses::EasyCurses;
use std::{cell::RefCell, rc::Rc};

pub use easycurses::Input as Key;

pub struct TuiInputSystem {
    easy: Rc<RefCell<EasyCurses>>,
}

impl TuiInputSystem {
    pub fn new(easy: Rc<RefCell<EasyCurses>>) -> Self {
        TuiInputSystem { easy }
    }
}

#[derive(SystemData)]
pub struct TuiInputSD<'s> {
    key_events: Write<'s, EventChannel<Key>>,
    screen_size: Write<'s, ScreenSize>,
}

impl<'s> System<'s> for TuiInputSystem {
    type SystemData = TuiInputSD<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut keys = Vec::new();
        let mut easy = self.easy.borrow_mut();
        for input in easy.get_input() {
            match input {
                Key::Character(_) => {
                    keys.push(input);
                }
                Key::KeyResize => {
                    let (height, width) = easy.get_row_col_count();

                    data.screen_size.width = width;
                    data.screen_size.height = height;
                }
                _ => {}
            }
        }
        data.key_events.iter_write(keys);
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        // res.insert(EventChannel::<Key>::new());
    }
}
