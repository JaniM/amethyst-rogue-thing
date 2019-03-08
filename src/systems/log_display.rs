use crate::{
    components::{BoardDisplay, LogDisplay},
    resources::{EventLog, LogEvents},
    tui::{Position, ScreenSize, TextBlock, TuiChannel, TuiEvent},
};
use amethyst::ecs::{prelude::*, SystemData as _};

#[derive(Default)]
pub struct LogDisplaySystem {
    tui_reader: Option<ReaderId<TuiEvent>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    log_events: Write<'s, LogEvents>,
    log: Write<'s, EventLog>,
    text_block: WriteStorage<'s, TextBlock>,
    log_display: ReadStorage<'s, LogDisplay>,
    screen_size: Read<'s, ScreenSize>,
    position: WriteStorage<'s, Position>,
    board: ReadStorage<'s, BoardDisplay>,
    tui_channel: Read<'s, TuiChannel>,
}

impl<'s> System<'s> for LogDisplaySystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = false;
        let mut log_pos = None;

        for event in data.tui_channel.read(self.tui_reader.as_mut().unwrap()) {
            match event {
                TuiEvent::ScreenSize { .. } => dirty = true,
                _ => {}
            }
        }

        data.log.events.reverse();
        while let Ok(line) = data.log_events.receiver().try_recv() {
            data.log.events.push(line.0);
            dirty = true;
        }
        data.log.events.reverse();

        if dirty {
            for (block, pos, _) in (
                &mut data.text_block,
                (&mut data.position).maybe(),
                &data.log_display,
            )
                .join()
            {
                if let Some(pos) = pos {
                    block.width = 50;
                    let new_x = (data.screen_size.width - block.width - 1).max(15);
                    let changed = pos.x != new_x;
                    pos.x = new_x;
                    block.width = (data.screen_size.width - pos.x - 1).min(50);
                    block.height = data.screen_size.height - pos.y - 1;
                    if changed {
                        log_pos = Some(*pos);
                    }
                }
                block.rows = [format!("({}, {})", block.width, block.height)]
                    .iter()
                    .chain(data.log.events.iter())
                    .chain(["".to_owned()].into_iter().cycle())
                    .map(|x| {
                        ("| ".to_owned() + x)
                            .chars()
                            .take(block.width as usize - 1)
                            .collect()
                    })
                    .take(block.height as usize)
                    .collect();
            }
        }

        if let Some(log_pos) = log_pos {
            for (block, pos, _) in (&data.text_block, &mut data.position, &data.board).join() {
                pos.x = log_pos.x / 2 - block.width / 2;
                pos.y = data.screen_size.height / 2 - block.height / 2;
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.tui_reader = Some(res.get_mut::<TuiChannel>().unwrap().register_reader());
    }
}
