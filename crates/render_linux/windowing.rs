use render_components::RenderComponent;
use render_events::{ClickDevice, Events};
use render_platform_options::{RenderMode, WindowOptions};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

use crate::renderer::Renderer;

#[derive(Default)]
struct App {
    window: Option<Box<Window>>,
    renderer: Option<Renderer>,
    base_component: Option<Box<dyn RenderComponent>>,
    window_options: WindowOptions,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title(self.window_options.title.clone())
            .with_inner_size(Size::Logical(LogicalSize {
                width: self.window_options.default_width as f64,
                height: self.window_options.default_height as f64,
            }));
        let window = event_loop
            .create_window(attributes)
            .expect("Could not create window");
        let window = Box::new(window);
        // Create a raw pointer to avoid move issues
        let window_ptr: *const Window = &*window;
        self.window = Some(window);
        let size = self.window.as_ref().unwrap().inner_size();
        self.renderer = Some(Renderer::setup(
            window_ptr,
            size,
            self.window_options.clone(),
        ));
        self.base_component
            .as_mut()
            .unwrap()
            .handle_event(Events::Resize {
                width: size.width,
                height: size.height,
            });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.base_component
                    .as_mut()
                    .unwrap()
                    .handle_event(Events::Resize {
                        width: size.width,
                        height: size.height,
                    });
            }
            WindowEvent::RedrawRequested => {
                self.renderer.as_mut().unwrap().clear();
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::CursorMoved {
                device_id: _device_id,
                position,
            } => {
                self.base_component
                    .as_mut()
                    .unwrap()
                    .handle_event(Events::Hover {
                        x: position.x as i32,
                        y: position.y as i32,
                    });
                render_events::update_mouse_position(position.x as i32, position.y as i32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed && button == MouseButton::Left {
                    let pos = render_events::get_mouse_position();
                    self.base_component
                        .as_mut()
                        .unwrap()
                        .handle_event(Events::PrimaryClick {
                            x: pos.0,
                            y: pos.1,
                            click_device: ClickDevice::Mouse,
                        });
                }
            }
            _ => {}
        }
    }
}

pub fn run(base_component: impl RenderComponent + 'static, window_options: WindowOptions) {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    match window_options.render_mode {
        RenderMode::HighPerformance => {
            event_loop.set_control_flow(ControlFlow::Poll);
        }
        RenderMode::LowPower => {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    let mut app = App {
        base_component: Some(Box::new(base_component)),
        window_options,
        ..Default::default()
    };
    event_loop
        .run_app(&mut app)
        .expect("Failed to run application");
}
