//! Window management and egui integration for simple applications.

use ::egui::Context;
use wgpu::RenderPass;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};

use crate::gpu::Gpu;

pub mod egui;
pub mod ui;
pub mod window;

#[derive(Clone)]
pub struct GraphicsCtx<'a> {
    pub gpu: &'a Gpu,
    pub window: &'a winit::window::Window,
}

pub trait Interactive {
    fn init(&mut self, _gcx: GraphicsCtx) {}
    fn render(&mut self, _gcx: GraphicsCtx, _render_pass: &mut RenderPass) {}
    fn ui(&mut self, _gcx: GraphicsCtx, _ctx: &Context) {}
    fn window_event(&mut self, _gcx: GraphicsCtx, _event: &WindowEvent) {}
    fn device_event(&mut self, _gcx: GraphicsCtx, _device_id: DeviceId, _event: &DeviceEvent) {}
}
