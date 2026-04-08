use render_events::Events;
use render_layout::{
    positioning::{Positioning, PositioningType},
    sizing::Sizing,
};

use crate::shapes::Shapes;

pub mod empty_component;
pub mod layout_solvable;
pub mod shapes;

pub trait RenderComponent {
    fn render(&'_ self) -> Vec<Shapes<'_>>;
    fn children(&'_ self) -> Option<Vec<Box<dyn RenderComponent>>> {
        None
    }
    fn handle_event(&mut self, event: Events) {
        match event {
            Events::Resize { width, height } => self.handle_resize(width, height),
            Events::Move { x, y } => self.handle_move(x, y),
            Events::Hover { x, y } =>
            {
                #[allow(unused_variables)]
                if let Some(children) = self.children() {
                    todo!(
                        "Handle hover event for children. Got hover at: ({}, {})",
                        x,
                        y
                    )
                }
            }
            _ => {
                todo!(
                    "Handle other events in RenderComponent. Got event: {:?}",
                    event
                )
            }
        }
    }
    // Resizing
    fn get_sizing(&'_ self) -> Sizing<'_>;
    fn resize(&mut self, width: u32, height: u32);
    fn handle_resize(&mut self, width: u32, height: u32) {
        self.resize(width, height);
        #[allow(unused_variables)]
        if let Some(children) = self.children() {
            todo!("Layout children")
        }
    }
    // Repositioning
    fn get_position(&'_ self) -> Positioning {
        Positioning::new(PositioningType::Auto, PositioningType::Auto)
    }
    fn reposition(&mut self, x: u32, y: u32);
    fn handle_move(&mut self, x: u32, y: u32) {
        self.reposition(x, y);
        #[allow(unused_variables)]
        if let Some(children) = self.children() {
            todo!("Reposition children")
        }
    }
}
