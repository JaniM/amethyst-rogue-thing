extern crate amethyst;

extern crate shred;
#[macro_use]
extern crate shred_derive;

extern crate crossbeam_channel;

extern crate termion;

mod bundles;
mod components;
mod data;
mod graphic;
mod play;
mod resources;
mod systems;

use crate::{
    bundles::{LiveBundle, TickBundle},
    play::PlayState,
};

use amethyst::{
    core::{transform::TransformBundle, ArcThreadPool, SystemBundle},
    ecs::{Dispatcher, DispatcherBuilder},
    input::{Bindings, Button, InputBundle},
    prelude::*,
    renderer::{DisplayConfig, DrawFlat, Pipeline, PosTex, RenderBundle, Stage, VirtualKeyCode},
    utils::application_root_dir,
};

pub struct CustomGameData<'a, 'b> {
    live_dispatcher: Dispatcher<'a, 'b>,
    tick_dispatcher: Dispatcher<'a, 'b>,
}

pub struct CustomGameDataBuilder<'a, 'b> {
    pub live: DispatcherBuilder<'a, 'b>,
    pub tick: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> Default for CustomGameDataBuilder<'a, 'b> {
    fn default() -> Self {
        CustomGameDataBuilder::new()
    }
}

impl<'a, 'b> CustomGameDataBuilder<'a, 'b> {
    pub fn new() -> Self {
        CustomGameDataBuilder {
            live: DispatcherBuilder::new(),
            tick: DispatcherBuilder::new(),
        }
    }

    pub fn with_live_bundle<B>(mut self, bundle: B) -> amethyst::Result<Self>
    where
        B: SystemBundle<'a, 'b>,
    {
        bundle
            .build(&mut self.live)
            .map_err(|err| amethyst::Error::Core(err))?;
        Ok(self)
    }

    pub fn with_tick_bundle<B>(mut self, bundle: B) -> amethyst::Result<Self>
    where
        B: SystemBundle<'a, 'b>,
    {
        bundle
            .build(&mut self.tick)
            .map_err(|err| amethyst::Error::Core(err))?;
        Ok(self)
    }
}

impl<'a, 'b> DataInit<CustomGameData<'a, 'b>> for CustomGameDataBuilder<'a, 'b> {
    fn build(self, world: &mut World) -> CustomGameData<'a, 'b> {
        // Get a handle to the `ThreadPool`.
        let pool = world.read_resource::<ArcThreadPool>().clone();

        let mut live_dispatcher = self.live.with_pool(pool.clone()).build();
        live_dispatcher.setup(&mut world.res);

        let mut tick_dispatcher = self.tick.with_pool(pool.clone()).build();
        tick_dispatcher.setup(&mut world.res);

        CustomGameData {
            live_dispatcher,
            tick_dispatcher,
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let path = format!("{}/resources/display_config.ron", application_root_dir());
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );

    let mut bindings = Bindings::new();
    bindings.insert_action_binding("up".to_owned(), vec![Button::Key(VirtualKeyCode::W)]);
    bindings.insert_action_binding("down".to_owned(), vec![Button::Key(VirtualKeyCode::S)]);
    bindings.insert_action_binding("left".to_owned(), vec![Button::Key(VirtualKeyCode::A)]);
    bindings.insert_action_binding("right".to_owned(), vec![Button::Key(VirtualKeyCode::D)]);
    bindings.insert_action_binding("wait".to_owned(), vec![Button::Key(VirtualKeyCode::X)]);

    let input_bundle = InputBundle::<String, String>::new().with_bindings(bindings);

    let game_data = CustomGameDataBuilder::default()
        .with_live_bundle(LiveBundle::default())?
        .with_live_bundle(RenderBundle::new(pipe, Some(config)))?
        .with_live_bundle(TransformBundle::new())?
        .with_live_bundle(input_bundle)?
        .with_tick_bundle(TickBundle::default())?;
    let mut game = Application::new("./", PlayState, game_data)?;

    game.run();

    Ok(())
}
