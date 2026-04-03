use crate::renderer::Renderer;

impl Renderer {
    pub(crate) fn reconfigure(&mut self, width: u32, height: u32) {
        let surface = self.surface.as_ref().unwrap();
        surface.configure(
            &self.device.as_ref().unwrap(),
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface
                    .get_capabilities(&self.adapter.as_ref().unwrap())
                    .formats[0],
                width: width,
                height: height,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![], // danger! do not know!
            },
        );
    }
}
