use std::sync::Arc;

use wgpu::{Adapter, Color, Device, Instance, Queue, Surface};
use winit::window::Window;

use crate::renderer::{rectangle::RectangleRenderer, text::TextRenderer};
pub(crate) mod reconfigure;
pub(crate) mod rectangle;
pub(crate) mod rendering;
pub(crate) mod setup;
pub(crate) mod text;

#[allow(unused)]
pub(crate) struct Renderer {
    instance: Option<Instance>,
    surface: Option<Surface<'static>>,
    adapter: Option<Adapter>,
    device: Option<Device>,
    queue: Option<Queue>,
    pub(crate) clear_color: Color,
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) rectangle_renderer: Option<RectangleRenderer>,
    pub(crate) text_renderer: Option<TextRenderer>,
}
