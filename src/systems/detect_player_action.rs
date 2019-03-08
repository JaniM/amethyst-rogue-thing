use crate::{
    data::{Direction, PlayerAction},
    resources::{LogEvents, PlayerActionResource},
    tui::Key,
};
use amethyst::{
    core::{shrev::EventChannel, timing::Time},
    ecs::{prelude::*, SystemData as _},
};

#[derive(Default)]
pub struct DetectPlayerActionSystem {
    reader: Option<ReaderId<Key>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    inputs: Read<'s, EventChannel<Key>>,
    action: Write<'s, PlayerActionResource>,
    time: Read<'s, Time>,
    log: Read<'s, LogEvents>,
}

impl<'s> System<'s> for DetectPlayerActionSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        use Direction::*;

        let mut action = None;

        for key in data.inputs.read(self.reader.as_mut().unwrap()) {
            match key {
                Key::Character('\u{1b}') => action = Some(PlayerAction::Quit),
                Key::Character('w') => action = Some(PlayerAction::Move(Up)),
                Key::Character('s') => action = Some(PlayerAction::Move(Down)),
                Key::Character('a') => action = Some(PlayerAction::Move(Left)),
                Key::Character('d') => action = Some(PlayerAction::Move(Right)),
                Key::Character('x') => action = Some(PlayerAction::Wait),
                x => {
                    data.log.send(format!("Unrecognized input: {:?}", x));
                }
            }
        }

        data.action.hold_delay -= data.time.delta_seconds();

        if action.is_some() && data.action.hold_delay <= 0.0 {
            data.action.action = action;
            data.action.hold_delay = 0.25;
        } else if action.is_none() {
            data.action.hold_delay = 0.0;
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        res.insert(EventChannel::<Key>::new());
        self.reader = Some(
            res.get_mut::<EventChannel<Key>>()
                .unwrap()
                .register_reader(),
        );
    }
}
