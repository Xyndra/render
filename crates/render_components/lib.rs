use render_events::Events;
use render_layout::Sizing;

pub mod empty_component;

#[derive(Debug)]
pub enum Shapes<'a> {
    Rectangle {
        sizing: Sizing<'a>,
        color: (u8, u8, u8),
        rounding: Option<u32>,
    },
    ConstrainedText {
        sizing: Sizing<'a>,
        content: String,
        font: String,
    },
}

pub trait RenderComponent {
    fn get_sizing(&'_ self) -> Sizing<'_>;
    fn render(&'_ self) -> Vec<Shapes<'_>>;
    fn children(&'_ self) -> Vec<Box<dyn RenderComponent>>;
    fn handle_event(&mut self, event: Events);
}

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
