use ::egui::Context;
use wgpu::{CommandEncoder, RenderPass};
use winit::window::Window;

use crate::gpu::Gpu;

pub mod egui;
pub mod window;

pub trait Interactive {
    fn init(&mut self, gpu: &Gpu, window: &Window) {}
    fn render(&mut self, gpu: &Gpu, render_pass: &mut RenderPass) {}
    fn ui(&mut self, ctx: &Context) {}
}
