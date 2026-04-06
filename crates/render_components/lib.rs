use render_events::Events;
use render_layout::Sizing;

use crate::shapes::Shapes;

pub mod empty_component;
pub mod layout_solvable;
pub mod shapes;

pub trait RenderComponent {
    fn get_sizing(&'_ self) -> Sizing<'_>;
    fn render(&'_ self) -> Vec<Shapes<'_>>;
    fn children(&'_ self) -> Option<Vec<Box<dyn RenderComponent>>> {
        None
    }
    fn handle_event(&mut self, event: Events) {
        match event {
            Events::Resize { width, height } => self.handle_resize(width, height),
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
    fn handle_resize(&mut self, width: u32, height: u32) {
        self.resize(width, height);
        #[allow(unused_variables)]
        if let Some(children) = self.children() {
            todo!("Layout children")
        }
    }
    fn resize(&mut self, width: u32, height: u32);
}
