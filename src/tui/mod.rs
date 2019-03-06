use amethyst::{core::shrev::EventChannel, ecs::prelude::*};

use std::{
    fs::File,
    io::{Read as _, Write as _},
    sync::Mutex,
};

pub use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Position { x, y }
    }
}

impl Component for Position {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl Default for Position {
    fn default() -> Self {
        Position::new(0, 0)
    }
}

pub struct TextBlock {
    pub rows: Vec<String>,
}

impl TextBlock {
    pub fn new<T, K>(rows: T) -> Self
    where
        T: IntoIterator<Item = K>,
        K: Into<String>,
    {
        TextBlock {
            rows: rows.into_iter().map(|x| x.into()).collect(),
        }
    }

    pub fn single_rpw<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        TextBlock::new(vec![text])
    }

    pub fn empty() -> Self {
        TextBlock::new(Vec::<String>::new())
    }
}

impl Component for TextBlock {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

pub struct RawTerminalRes(pub RawTerminal<AlternateScreen<File>>);

#[derive(Default)]
pub struct TuiRenderSystem {
    position_reader: Option<ReaderId<ComponentEvent>>,
    textblock_reader: Option<ReaderId<ComponentEvent>>,
}

#[derive(SystemData)]
pub struct TuiRenderSD<'s> {
    raw: WriteExpect<'s, RawTerminalRes>,
    position: ReadStorage<'s, Position>,
    text_block: ReadStorage<'s, TextBlock>,
}

impl<'s> System<'s> for TuiRenderSystem {
    type SystemData = TuiRenderSD<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = false;

        for event in data
            .position
            .channel()
            .read(self.position_reader.as_mut().unwrap())
            .chain(
                data.text_block
                    .channel()
                    .read(self.textblock_reader.as_mut().unwrap()),
            )
        {
            match event {
                ComponentEvent::Modified(_id) | ComponentEvent::Inserted(_id) => {
                    dirty = true;
                }
                ComponentEvent::Removed(_id) => {
                    dirty = true;
                }
            }
        }

        if !dirty {
            return;
        }

        let out = &mut data.raw.0;
        write!(
            out,
            "{clear}{hide}",
            clear = termion::clear::All,
            hide = termion::cursor::Hide
        )
        .expect("Write failed");

        for (pos, block) in (&data.position, &data.text_block).join() {
            for (i, row) in block.rows.iter().enumerate() {
                write!(
                    out,
                    "{}{}",
                    termion::cursor::Goto(pos.x + 1, pos.y + 1 + i as u16),
                    row
                )
                .expect("Write failed");
            }
        }
        out.flush().ok();
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        res.insert(RawTerminalRes(
            AlternateScreen::from(termion::get_tty().unwrap())
                .into_raw_mode()
                .unwrap(),
        ));
        self.position_reader = Some(WriteStorage::<Position>::fetch(&res).register_reader());
        self.textblock_reader = Some(WriteStorage::<TextBlock>::fetch(&res).register_reader());
    }
}

pub struct TerminalInputRes(pub Mutex<termion::AsyncReader>);

pub struct TuiInputSystem;

#[derive(SystemData)]
pub struct TuiInputSD<'s> {
    input: WriteExpect<'s, TerminalInputRes>,
    key_events: WriteExpect<'s, EventChannel<Key>>,
}

impl<'s> System<'s> for TuiInputSystem {
    type SystemData = TuiInputSD<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut buffer: [u8; 10] = [0; 10];
        let mut stream = data.input.0.lock().unwrap();
        let length = stream.read(&mut buffer).unwrap();
        let keys = buffer[..length]
            .keys()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        data.key_events.iter_write(keys);
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        res.insert(TerminalInputRes(Mutex::new(termion::async_stdin())));
        res.insert(EventChannel::<Key>::new());
    }
}

pub fn cleanup(world: &mut World) {
    let raw = &mut world.write_resource::<RawTerminalRes>().0;
    write!(
        raw,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Show
    )
    .expect("Screen clear failed");
}
