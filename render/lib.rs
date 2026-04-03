use render_components::RenderComponent;
use render_platform_options::WindowOptions;

pub fn run_default(basecomponent: impl RenderComponent + 'static) {
    run(basecomponent, None);
}

pub fn run(basecomponent: impl RenderComponent + 'static, window_options: Option<WindowOptions>) {
    #[allow(unused_assignments)]
    let mut resolved = false;
    let window_options = window_options.unwrap_or_default();
    #[cfg(target_os = "linux")]
    {
        render_linux::windowing::run(basecomponent, window_options);
        resolved = true;
    }

    if !resolved {
        panic!("Unsupported platform");
    }
}
