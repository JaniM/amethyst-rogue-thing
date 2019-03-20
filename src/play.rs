use amethyst::{core::transform::Parent, ecs::Entity, prelude::*};

use crate::{
    components::*,
    data::*,
    resources::*,
    tui::{
        border::Border,
        centering::Centered,
        components::*,
        stacking::{StackingContext, StackingRule},
        visibility_relation::VisibleIfChildIs,
    },
    CustomGameData,
};

pub struct PlayState;

impl<'a, 'b> State<CustomGameData<'a, 'b>, StateEvent> for PlayState {
    fn on_start(&mut self, data: StateData<CustomGameData>) {
        let world = data.world;
        world.register::<Character>();

        world.add_resource(WorldMap::new(20, 20));

        let stack = world
            .create_entity()
            .with(StackingContext::horizontal())
            .with(Position::new(0, 0))
            .build();

        let board_container = world
            .create_entity()
            .with(Parent { entity: stack })
            .with(StackingRule::new().min_width(15).min_height(15))
            .build();

        let board = world
            .create_entity()
            .with(Parent {
                entity: board_container,
            })
            .with(Centered::new(true, true))
            .with(TextBlock::new((0..10).map(|_| ".".repeat(10)), 10, 10))
            .build();

        let rhs = world
            .create_entity()
            .with(Parent { entity: stack })
            .with(StackingRule::new().max_width(80).min_width(50).flex(2))
            .with(StackingContext::vertical())
            .build();

        let inventory_border = world
            .create_entity()
            .with(Parent { entity: rhs })
            .with(StackingRule::new())
            .with(Border::new().left())
            .build();

        world
            .create_entity()
            .with(Parent {
                entity: inventory_border,
            })
            .with(Position::new(2, 0))
            .with(InventoryDisplay::new(InventoryDisplayKind::Own))
            .build();

        let ground_border = world
            .create_entity()
            .with(Parent { entity: rhs })
            .with(StackingRule::new())
            .with(Border::new().top().left())
            .with(VisibleIfChildIs)
            .build();

        world
            .create_entity()
            .with(Parent {
                entity: ground_border,
            })
            .with(Position::new(2, 1))
            .with(InventoryDisplay::new(InventoryDisplayKind::Ground))
            .build();

        let log_border = world
            .create_entity()
            .with(Parent { entity: rhs })
            .with(StackingRule::new())
            .with(Border::new().top().left())
            .build();

        world
            .create_entity()
            .with(Parent { entity: log_border })
            .with(Position::new(2, 1))
            .with(LogDisplay)
            .build();

        world.add_resource(Board(Some(board)));

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
        .with(ZLevel::new(1))
        .with(Inventory::new(vec![Item::new(vec![
            ItemPart::Name("Test weapon".into()),
            ItemPart::Damage(-1),
        ])]))
        .build();

    // Character + WprldPosition: recognized as a character in the world
    // Character + WorldPosition + Team + Health: recognized as a character that can be attacked
    // PlayerControlledCharacter: reads player input
    // Parent + Position + TextBlock: recognized as a renderable thing
    // Inventory: has items, can pick up items, drops them when dead
    // etc

    world.add_resource(PlayerEntity(Some(entity)));
}

pub fn initialise_enemy(world: &mut World) {
    use rand::Rng;
    let board = world.read_resource::<Board>().0.unwrap();
    let mut rng = rand::thread_rng();
    world
        .create_entity()
        .with(Character)
        .with(WorldPosition::new(8, 8))
        .with(Parent { entity: board })
        .with(Team(1))
        .with(AggressiveAI::new(&[0]))
        .with(Health::new(5))
        .with(Position::default())
        .with(TextBlock::single_row("c"))
        .with(ZLevel::new(1))
        .with(Inventory::new(vec![Item::new(vec![
            ItemPart::Name(["Wooden nail", "Shoe", "???"][rng.gen_range(0, 3)].into()),
            ItemPart::Damage(rng.gen_range(0, 5)),
        ])]))
        .with(Named::new("Enemy"))
        .build();
}

pub fn initialise_item<T: Builder>(
    builder: T,
    board: Entity,
    position: WorldPosition,
    item: Item,
) -> T {
    builder
        .with(position)
        .with(Parent { entity: board })
        .with(Position::new(position.x, position.y))
        .with(TextBlock::single_row("*"))
        .with(item)
}
