use render_components::RenderComponent;
use render_events::{ClickDevice, Events};
use render_platform_options::{RenderMode, WindowOptions};
use wgpu::{Adapter, CurrentSurfaceTexture, Device, Instance, InstanceDescriptor, Queue, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

#[derive(Default)]
struct App {
    window: Option<Box<Window>>,
    instance: Option<Instance>,
    surface: Option<Surface<'static>>,
    adapter: Option<Adapter>,
    device: Option<Device>,
    queue: Option<Queue>,
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
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        // SAFETY: window_ptr remains valid because window hasn't been moved yet
        let surface = instance
            .create_surface(unsafe { &*window_ptr })
            .expect("Failed to create surface");
        self.window = Some(window);
        let high_performance = self.window_options.render_mode == RenderMode::HighPerformance;
        let power_preference = if high_performance {
            wgpu::PowerPreference::HighPerformance
        } else {
            wgpu::PowerPreference::LowPower
        };
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }))
            .expect("Failed to find a suitable GPU adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
            ..Default::default()
        }))
        .expect("Failed to create device");
        let size = self.window.as_ref().unwrap().inner_size();
        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface.get_capabilities(&adapter).formats[0],
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![], // danger! do not know!
            },
        );
        self.base_component
            .as_mut()
            .unwrap()
            .handle_event(Events::Resize {
                width: size.width,
                height: size.height,
            });
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.instance = Some(instance);
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
                // todo!
                let output = self.surface.as_ref().unwrap().get_current_texture();
                #[allow(unused_assignments)]
                let mut surface_texture: Option<wgpu::SurfaceTexture> = None;
                match output {
                    CurrentSurfaceTexture::Success(texture) => {
                        surface_texture = Some(texture);
                    }
                    _ => {
                        // todo! handle all the other cases
                        eprintln!("Failed to acquire texture");
                        self.window.as_ref().unwrap().request_redraw();
                        return;
                    }
                }
                let view = surface_texture
                    .as_ref()
                    .unwrap()
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = self.device.as_mut().unwrap().create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    },
                );

                {
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        ..Default::default()
                    });
                }
                self.queue
                    .as_ref()
                    .unwrap()
                    .submit(std::iter::once(encoder.finish()));
                surface_texture.unwrap().present();

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
