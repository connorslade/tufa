use wgpu::TextureFormat;

pub mod bindings;
pub mod gpu;
#[cfg(feature = "interactive")]
pub mod interactive;
pub mod misc;
pub mod pipeline;

pub mod export {
    #[cfg(feature = "interactive")]
    pub use {egui, winit};
    pub use {encase, nalgebra, wgpu};
}

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth24PlusStencil8;
