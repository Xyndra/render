use render::run_default;
use render_components::empty_component::EmptyComponent;
use render_layout::Layoutable;

fn main() {
    let mut component = EmptyComponent::default();
    component.color = (255, 0, 0, 255); // Set the color to red
    component.children = component.children(); // Generate the child rectangle with the specified color
    run_default(component);
}
