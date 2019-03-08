pub mod blink;
pub mod components;
pub mod input;
pub mod render;

pub use self::{
    components::*,
    input::{Key, TuiInputSystem},
    render::{BlinkSystem, ScreenSize, TuiRenderSystem},
};

use std::{cell::RefCell, rc::Rc};

use amethyst::{
    core::{Parent, SystemBundle},
    ecs::DispatcherBuilder,
};
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
        builder.add(BlinkSystem::new(), "blink_system", self.dep);
        builder.add_thread_local(TuiRenderSystem::new(easy.clone()));
        builder.add_thread_local(TuiInputSystem::new(easy));
        Ok(())
    }
}
