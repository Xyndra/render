use render_events::Events;
use render_layout::Layout;

use crate::RenderComponent;

pub trait LayoutSolvable {
    fn get_layout(&'_ self) -> Layout<'_>;
    fn set_resolved_layout(&mut self, layout: Layout);
}

impl<T: RenderComponent> LayoutSolvable for T {
    fn get_layout(&'_ self) -> Layout<'_> {
        Layout {
            sizing: self.get_sizing(),
            positioning: self.get_position(),
        }
    }

    fn set_resolved_layout(&mut self, layout: Layout) {
        self.handle_event(Events::Resize {
            width: layout
                .sizing
                .resolved_width
                .expect("Layout must be resolved before setting"),
            height: layout
                .sizing
                .resolved_height
                .expect("Layout must be resolved before setting"),
        });
        self.handle_event(Events::Move {
            x: layout
                .positioning
                .resolved_x
                .expect("Layout must be resolved before setting. Missing resolved_x"),
            y: layout
                .positioning
                .resolved_y
                .expect("Layout must be resolved before setting. Missing resolved_y"),
        });
    }
}
