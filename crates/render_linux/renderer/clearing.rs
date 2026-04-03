use crate::renderer::Renderer;
use wgpu::CurrentSurfaceTexture;

impl Renderer {
    pub(crate) fn clear(&mut self) {
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
                return;
            }
        }
        let view = surface_texture
            .as_ref()
            .unwrap()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.device
                .as_mut()
                .unwrap()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

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
    }
}
