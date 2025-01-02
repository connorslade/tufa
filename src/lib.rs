use wgpu::TextureFormat;

pub mod buffer;
pub mod gpu;
mod misc;
pub mod pipeline;
pub mod window;

pub mod export {
    pub use {encase, nalgebra, wgpu, winit};
}

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

// todo storage vs uniform buffers
