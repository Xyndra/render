use render::run_default;
use render_layout::Layoutable;
pub mod app;
pub mod logic;
pub mod state;
pub mod topbar;
use app::App;

fn main() {
    let mut component = App::default();
    component.children = component.children(); // Generate the child rectangle with the specified color
    run_default(component);
}

#[cfg(test)]
use render::test;
#[test]
fn layouting() {
    let mut component = App::default();
    component.children = component.children(); // Generate the child rectangle with the specified color
    test(component, None);
}
