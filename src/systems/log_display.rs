use crate::{
    components::LogDisplay,
    resources::{EventLog, LogEvents},
    tui::TextBlock,
};
use amethyst::ecs::prelude::*;

pub struct LogDisplaySystem;

#[derive(SystemData)]
pub struct SystemData<'s> {
    log_events: Write<'s, LogEvents>,
    log: Write<'s, EventLog>,
    text_block: WriteStorage<'s, TextBlock>,
    log_display: ReadStorage<'s, LogDisplay>,
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
        data.log.events.truncate(10);

        if dirty {
            for (block, _) in (&mut data.text_block, &data.log_display).join() {
                block.rows = data.log.events.clone();
            }
        }
    }
}
