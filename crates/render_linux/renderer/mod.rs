use wgpu::{Adapter, Device, Instance, Queue, Surface};
pub(crate) mod clearing;
pub(crate) mod setup;

#[allow(unused)]
pub(crate) struct Renderer {
    instance: Option<Instance>,
    surface: Option<Surface<'static>>,
    adapter: Option<Adapter>,
    device: Option<Device>,
    queue: Option<Queue>,
}
