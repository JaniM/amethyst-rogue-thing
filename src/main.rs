extern crate amethyst;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate crossbeam_channel;
extern crate easycurses;
extern crate hibitset;
extern crate rand;
extern crate specs_hierarchy;

mod bundles;
mod components;
mod data;
mod play;
mod resources;
mod specs_ext;
mod system_chain;
mod systems;
mod tui;

use crate::{
    bundles::{LiveBundle, TickBundle},
    play::PlayState,
};

use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, ArcThreadPool, SystemBundle},
    ecs::{Dispatcher, DispatcherBuilder},
    prelude::*,
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

    let game_data = CustomGameDataBuilder::default()
        .with_live_bundle(LiveBundle::default())?
        .with_tick_bundle(TickBundle::default())?;
    let mut game = Application::build("./", PlayState)?
        .with_frame_limit(FrameRateLimitStrategy::Sleep, 30)
        .build(game_data)?;

    game.run();

    Ok(())
}
