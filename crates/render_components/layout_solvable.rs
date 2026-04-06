use render_events::Events;
use render_layout::Sizing;

use crate::RenderComponent;

pub trait LayoutSolvable {
    fn get_layout(&'_ self) -> Sizing<'_>;
    fn set_resolved_layout(&mut self, layout: Sizing);
}

impl<T: RenderComponent> LayoutSolvable for T {
    fn get_layout(&'_ self) -> Sizing<'_> {
        self.get_sizing()
    }

    fn set_resolved_layout(&mut self, layout: Sizing) {
        self.handle_event(Events::Resize {
            width: layout
                .resolved_width
                .expect("Layout must be resolved before setting"),
            height: layout
                .resolved_height
                .expect("Layout must be resolved before setting"),
        });
    }
}
