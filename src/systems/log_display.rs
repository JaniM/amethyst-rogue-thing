use crate::{
    components::LogDisplay,
    resources::{EventLog, LogEvents, TurnCounter},
    tui::TextBlock,
};
use amethyst::{core::Time, ecs::prelude::*};

#[derive(Default)]
pub struct LogDisplaySystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    log_events: Write<'s, LogEvents>,
    log: Write<'s, EventLog>,
    text_block: WriteStorage<'s, TextBlock>,
    log_display: ReadStorage<'s, LogDisplay>,
    time: Read<'s, Time>,
    turn: Read<'s, TurnCounter>,
}

impl<'s> System<'s> for LogDisplaySystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = false;
        data.log.events.reverse();
        while let Ok(line) = data.log_events.receiver().try_recv() {
            data.log.events.push(line.0);
            dirty = true;
        }
        data.log.events.reverse();

        if dirty {
            for (block, _) in (&mut data.text_block, &data.log_display).join() {
                block.rows = ["".to_owned()]
                    .iter()
                    .chain(data.log.events.iter())
                    .chain(["".to_owned()].into_iter().cycle())
                    .cloned()
                    .take(block.height as usize + 50)
                    .collect();
            }
        }
        for (block, _) in (&mut data.text_block, &data.log_display).join() {
            if block.rows.len() > 0 {
                block.rows[0] =
                    format!("Turn {}, delta {}", data.turn.0, data.time.delta_seconds());
            }
        }
    }
}
