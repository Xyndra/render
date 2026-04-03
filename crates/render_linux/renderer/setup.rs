use crate::renderer::Renderer;
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
        Self {
            instance: Some(instance),
            surface: Some(surface),
            adapter: Some(adapter),
            device: Some(device),
            queue: Some(queue),
        }
    }
}
