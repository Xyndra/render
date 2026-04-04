use std::sync::Arc;

use wgpu::{Adapter, Device, Instance, Queue, Surface};
use winit::window::Window;
pub(crate) mod clearing;
pub(crate) mod reconfigure;
pub(crate) mod setup;

#[allow(unused)]
pub(crate) struct Renderer {
    instance: Option<Instance>,
    surface: Option<Surface<'static>>,
    adapter: Option<Adapter>,
    device: Option<Device>,
    queue: Option<Queue>,
    pub(crate) clear_color: wgpu::Color,
    pub(crate) window: Option<Arc<Window>>,
}
