pub mod blink;
pub mod border;
pub mod centering;
pub mod components;
pub mod event;
pub mod input;
pub mod render;
pub mod stacking;
pub mod visibility_relation;

pub use self::{
    border::Border,
    components::*,
    event::{TuiChannel, TuiEvent},
    input::Key,
    render::{Parent, ParentHierarchy, ScreenSize},
    visibility_relation::VisibleIfChildIs,
};

use self::{
    border::BorderSystem,
    centering::CenteringSystem,
    event::TuiEventSystem,
    input::TuiInputSystem,
    render::{BlinkSystem, TuiRenderSystem},
    stacking::StackingSystem,
    visibility_relation::VisibilityRelationSystem,
};

use std::{cell::RefCell, rc::Rc};

use amethyst::{core::SystemBundle, ecs::DispatcherBuilder};
use easycurses::EasyCurses;
use specs_hierarchy::HierarchySystem;

#[derive(Default)]
pub struct TuiBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> TuiBundle<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    #[allow(dead_code)]
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c> SystemBundle<'a, 'b> for TuiBundle<'c> {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> amethyst::core::Result<()> {
        let easy = Rc::new(RefCell::new(EasyCurses::initialize_system().unwrap()));
        builder.add(
            HierarchySystem::<Parent>::new(),
            "parent_hierarchy_system",
            self.dep,
        );
        builder.add(
            TuiEventSystem::new(),
            "tui_event_system",
            &["parent_hierarchy_system"],
        );
        builder.add(
            VisibilityRelationSystem::default(),
            "tui_visibility_relation_system",
            &["tui_event_system"],
        );
        builder.add(
            StackingSystem::default(),
            "tui_stacking_system",
            &["tui_visibility_relation_system"],
        );
        builder.add(
            CenteringSystem::default(),
            "tui_centering_system",
            &["tui_event_system"],
        );
        builder.add(
            BorderSystem::default(),
            "tui_border_system",
            &["tui_stacking_system"],
        );
        builder.add(BlinkSystem::new(), "blink_system", self.dep);
        builder.add_thread_local(TuiRenderSystem::new(easy.clone()));
        builder.add_thread_local(TuiInputSystem::new(easy));
        Ok(())
    }
}
