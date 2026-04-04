use crate::renderer::Renderer;
use wgpu::{CurrentSurfaceTexture, LoadOp, StoreOp};

impl Renderer {
    pub(crate) fn clear(&mut self) {
        let output = self.surface.as_ref().unwrap().get_current_texture();
        #[allow(unused_assignments)]
        let mut surface_texture: Option<wgpu::SurfaceTexture> = None;
        #[allow(unused_assignments)]
        let mut should_reconfigure = false;
        match output {
            CurrentSurfaceTexture::Success(texture) => {
                surface_texture = Some(texture);
            }
            CurrentSurfaceTexture::Suboptimal(texture) => {
                eprintln!("Surface is suboptimal, but acquired texture");
                surface_texture = Some(texture);
                should_reconfigure = true;
            }
            CurrentSurfaceTexture::Timeout => {
                eprintln!("Timeout while acquiring texture, skipping frame");
                return;
            }
            CurrentSurfaceTexture::Occluded => {
                #[cfg(debug_assertions)]
                eprintln!("Window is occluded, skipping frame");
                return;
            }
            CurrentSurfaceTexture::Outdated => {
                #[cfg(debug_assertions)]
                eprintln!("Surface is outdated, reconfiguring and skipping frame");
                self.reconfigure(
                    self.window.as_ref().unwrap().inner_size().width,
                    self.window.as_ref().unwrap().inner_size().height,
                );
                return;
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
                        load: LoadOp::Clear(self.clear_color),
                        store: StoreOp::Store,
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
        // todo! actually render something instead of just clearing
        self.window.as_ref().unwrap().pre_present_notify();
        surface_texture.unwrap().present();
        if should_reconfigure {
            self.reconfigure(
                self.window.as_ref().unwrap().inner_size().width,
                self.window.as_ref().unwrap().inner_size().height,
            );
        }
    }
}
