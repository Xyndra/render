use render_components::RenderComponent;

pub fn run(basecomponent: impl RenderComponent) {
    #[allow(unused_assignments)]
    let mut resolved = false;
    #[cfg(target_os = "linux")]
    {
        render_linux::windowing::run(basecomponent);
        resolved = true;
    }

    if !resolved {
        panic!("Unsupported platform");
    }
}
