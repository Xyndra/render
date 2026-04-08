use crate::renderer::{Renderer, rectangle::RectangleRenderer};
use render_platform_options::{RenderMode, WindowOptions};
use wgpu::{Instance, InstanceDescriptor};
use winit::{dpi::PhysicalSize, window::Window};

impl Renderer {
    pub fn setup(
        window_ptr: *const Window,
        size: PhysicalSize<u32>,
        window_options: WindowOptions,
    ) -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        // SAFETY: window_ptr remains valid because window hasn't been moved yet
        let surface = instance
            .create_surface(unsafe { &*window_ptr })
            .expect("Failed to create surface");
        let high_performance = window_options.render_mode == RenderMode::HighPerformance;
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
        let clear_color = wgpu::Color {
            r: window_options.clear_color.0 as f64 / 255.0,
            g: window_options.clear_color.1 as f64 / 255.0,
            b: window_options.clear_color.2 as f64 / 255.0,
            a: 1.0,
        };
        // Get surface texture format for creating RectangleRenderer
        let texture_format = surface.get_capabilities(&adapter).formats[0];

        let mut renderer = Self {
            instance: Some(instance),
            surface: Some(surface),
            adapter: Some(adapter),
            device: Some(device),
            queue: Some(queue),
            clear_color,
            window: None,
            rectangle_renderer: None,
        };

        // Initialize rectangle renderer
        let device = renderer.device.as_ref().unwrap();
        let rectangle_renderer = RectangleRenderer::new(device, texture_format);
        renderer.rectangle_renderer = Some(rectangle_renderer);

        renderer.reconfigure(size.width, size.height);
        renderer
    }
}
