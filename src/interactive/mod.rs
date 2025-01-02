use ::egui::Context;
use wgpu::RenderPass;

use crate::gpu::Gpu;

pub mod egui;
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
}
