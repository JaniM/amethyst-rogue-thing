use amethyst::{
    core::transform::Parent,
    ecs::{SystemData, WriteStorage},
    prelude::*,
};

use crate::{components::*, data::PlayerAction, resources::*, tui::components::*, CustomGameData};

pub struct PlayState;

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for PlayState {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        let world = data.world;
        world.register::<Character>();

        world.add_resource(WorldMap::new(10, 10));

        let reader = WriteStorage::<WorldPosition>::fetch(&world.res).register_reader();
        world.add_resource(WorldPositionReader(reader));

        let board = world
            .create_entity()
            .with(Position::new(2, 1))
            .with(TextBlock::new((0..10).map(|_| ".........."), 10, 10))
            .with(BoardDisplay)
            .build();

        world.add_resource(Board(Some(board)));

        world
            .create_entity()
            .with(Position::new(16, 1))
            .with(TextBlock::empty(50, 10))
            .with(LogDisplay)
            .build();

        initialise_player(world);
        initialise_enemy(world);

        data.data.tick_dispatcher.dispatch(&world.res);
    }

    fn handle_event(
        &mut self,
        _: StateData<CustomGameData>,
        _event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        Trans::None
    }

    fn update(
        &mut self,
        data: StateData<CustomGameData>,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.world.write_resource::<PlayerActionResource>().action = None;
        data.data.live_dispatcher.dispatch(&data.world.res);
        if let Some(act) = &data.world.read_resource::<PlayerActionResource>().action {
            if *act == PlayerAction::Quit {
                return Trans::Pop;
            }
            data.data.tick_dispatcher.dispatch(&data.world.res);
        }
        Trans::None
    }
}

fn initialise_player(world: &mut World) {
    let board = world.read_resource::<Board>().0.unwrap();
    let entity = world
        .create_entity()
        .with(Character)
        .with(WorldPosition::new(1, 1))
        .with(Parent { entity: board })
        .with(PlayerControlledCharacter)
        .with(Team(0))
        .with(Health::new(10))
        .with(Position::default())
        .with(TextBlock::single_row("@"))
        .with(Named::new("Player"))
        .with(Blink::new(0.5))
        .build();

    world.add_resource(PlayerEntity(Some(entity)));
}

pub fn initialise_enemy(world: &mut World) {
    let board = world.read_resource::<Board>().0.unwrap();
    world
        .create_entity()
        .with(Character)
        .with(WorldPosition::new(5, 5))
        .with(Parent { entity: board })
        .with(Team(1))
        .with(AggressiveAI::new(&[0]))
        .with(Health::new(5))
        .with(Position::default())
        .with(TextBlock::single_row("c"))
        .with(Named::new("Enemy"))
        .build();
}
