use crate::{DEPTH_TEXTURE_FORMAT, TEXTURE_FORMAT};

pub trait TextureFormat {
    fn as_format() -> wgpu::TextureFormat;
}

/// Rgba8Unorm
pub struct Rgba8;
impl TextureFormat for Rgba8 {
    fn as_format() -> wgpu::TextureFormat {
        TEXTURE_FORMAT
    }
}

/// Depth24PlusStencil8
pub struct Depth;
impl TextureFormat for Depth {
    fn as_format() -> wgpu::TextureFormat {
        DEPTH_TEXTURE_FORMAT
    }
}
