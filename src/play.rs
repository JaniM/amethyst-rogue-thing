use amethyst::{
    core::transform::Transform,
    ecs::{SystemData, WriteStorage},
    input::{is_close_requested, is_key_down},
    prelude::*,
    renderer::{Camera, Projection, ScreenDimensions, VirtualKeyCode},
};

use crate::{
    components::*,
    graphic::{create_colour_material_static, create_mesh_static, generate_rectangle_vertices},
    resources::{PlayerActionResource, PlayerEntity, WorldMap, WorldPositionReader},
    CustomGameData,
};

pub struct PlayState;

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for PlayState {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        let world = data.world;
        world.register::<Character>();

        world.add_resource(WorldMap::new(10, 10));

        let reader = WriteStorage::<WorldPosition>::fetch(&world.res).register_reader();
        world.add_resource(WorldPositionReader(reader));

        initialise_camera(world);
        initialise_player(world);
        initialise_enemy(world);
    }

    fn handle_event(
        &mut self,
        _: StateData<CustomGameData>,
        event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(
        &mut self,
        data: StateData<CustomGameData>,
    ) -> Trans<CustomGameData<'a, 'b>, StateEvent> {
        data.world.write_resource::<PlayerActionResource>().action = None;
        data.data.live_dispatcher.dispatch(&data.world.res);
        if data
            .world
            .read_resource::<PlayerActionResource>()
            .action
            .is_some()
        {
            data.data.tick_dispatcher.dispatch(&data.world.res);
        }
        Trans::None
    }
}

fn initialise_camera(world: &mut World) {
    let dim = {
        let scr = world.read_resource::<ScreenDimensions>();
        (scr.width(), scr.height())
    };
    let mut transform = Transform::default();
    transform.set_z(1.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0, dim.0, 0.0, dim.1,
        )))
        .with(transform)
        .build();
}

fn initialise_player(world: &mut World) {
    let mesh = create_mesh_static(world, generate_rectangle_vertices(0.0, 0.0, 50.0, 50.0));
    let material = create_colour_material_static(world, [0.0, 0.0, 1.0, 1.0]);
    let entity = world
        .create_entity()
        .with(Transform::default())
        .with(mesh)
        .with(material)
        .with(Character)
        .with(WorldPosition::new(1, 1))
        .with(PlayerControlledCharacter)
        .with(Team(0))
        .with(Health::new(10))
        .with(AnimateMovement::with_speed(0.1))
        .build();

    world.add_resource(PlayerEntity(Some(entity)));
}

pub fn initialise_enemy(world: &mut World) {
    let mesh = create_mesh_static(world, generate_rectangle_vertices(0.0, 0.0, 50.0, 50.0));
    let material = create_colour_material_static(world, [1.0, 0.0, 0.0, 1.0]);
    world
        .create_entity()
        .with(Transform::default())
        .with(mesh)
        .with(material)
        .with(Character)
        .with(WorldPosition::new(5, 5))
        .with(Team(1))
        .with(AggressiveAI::new(&[0]))
        .with(Health::new(5))
        .with(AnimateMovement::with_speed(0.1))
        .build();
}
