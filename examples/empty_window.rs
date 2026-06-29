use render::run_default;
use render_components::EmptyComponent;
use render_layout::Layoutable;

fn main() {
    let mut component = EmptyComponent::default();
    component.color = (255, 0, 0, 255); // Set the color to red
    component.children = component.children(); // Generate the child rectangle with the specified color
    run_default(component);
}

#[cfg(test)]
use render::test;
#[test]
fn layouting() {
    let mut component = EmptyComponent::default();
    component.color = (255, 0, 0, 255); // Set the color to red
    component.children = component.children(); // Generate the child rectangle with the specified color
    test(component, None);
}
