use crate::{RenderComponent, Shapes};
use render_events::Events;
use render_layout::{Sizing, SizingType};

#[derive(Default)]
pub struct EmptyComponent {
    width: u32,
    height: u32,
}
impl RenderComponent for EmptyComponent {
    fn get_sizing(&'_ self) -> Sizing<'_> {
        Sizing::new(SizingType::Grow(1), SizingType::Grow(1))
    }

    fn render(&'_ self) -> Vec<Shapes<'_>> {
        vec![Shapes::Rectangle {
            sizing: Sizing::new(SizingType::Grow(1), SizingType::Grow(1)),
            color: (255, 255, 0),
            rounding: None,
        }]
    }

    fn children(&'_ self) -> Vec<Box<dyn RenderComponent>> {
        vec![]
    }

    fn handle_event(&mut self, event: Events) {
        match event {
            Events::Resize { width, height } => {
                self.width = width;
                self.height = height;
            }
            _ => {}
        }
    }
}
