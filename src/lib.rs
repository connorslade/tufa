use wgpu::TextureFormat;

pub mod buffer;
pub mod gpu;
pub mod interactive;
mod misc;
pub mod pipeline;

pub mod export {
    pub use {egui, encase, nalgebra, wgpu, winit};
}

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

// todo storage vs uniform buffers
