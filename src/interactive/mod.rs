use ::egui::Context;
use wgpu::RenderPass;
use winit::window::Window;

use crate::gpu::Gpu;

pub mod egui;
pub mod window;

pub trait Interactive {
    fn init(&mut self, _gpu: &Gpu, _window: &Window) {}
    fn render(&mut self, _gpu: &Gpu, _render_pass: &mut RenderPass) {}
    fn ui(&mut self, _ctx: &Context) {}
}
