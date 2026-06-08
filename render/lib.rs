use std::{fs::File, io::Write};

use render_components::primitives::primitve_from_any;
use render_events::Events;
use render_layout::EventHandler;
use render_platform_options::WindowOptions;

pub fn run_default(basecomponent: impl EventHandler + 'static) {
    run(basecomponent, None);
}

pub fn run(basecomponent: impl EventHandler + 'static, window_options: Option<WindowOptions>) {
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

/// Utility for testing changes in the layouting engine
pub fn test(basecomponent: impl EventHandler + 'static, window_options: Option<WindowOptions>) {
    let mut base_component = Box::new(basecomponent);
    let window_options = window_options.unwrap_or_default();
    base_component.handle_event(Events::Resize {
        width: window_options.default_width,
        height: window_options.default_height,
    });
    let dpi = 96; // Standard DPI for testing
    let shapes = base_component.layout(&|any| primitve_from_any(any), dpi);
    let exe_path = std::env::current_exe().expect("Failed to get exe path");
    let exe_name = exe_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split("-")
        .next()
        .unwrap()
        .to_owned();
    let path = exe_path.with_file_name(exe_name + ".layout");
    if path.exists() {
        // if file exists, compare its contents with the new layout output
        let existing_layout =
            std::fs::read_to_string(path).expect("Failed to read layout output file");
        let new_layout = format!("{:#?}", shapes);
        assert_eq!(existing_layout, new_layout, "Layout output has changed");
    } else {
        // if file does not exist, create it
        let mut file = File::create(path.clone()).expect("Failed to create layout output file");
        file.write_all(format!("{:#?}", shapes).as_bytes())
            .expect("Failed to write layout output");
        println!(
            "Layout output file created at {}. Please verify its contents and run the test again.",
            path.display()
        );
    }
}
