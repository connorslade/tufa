pub mod buffer;
pub mod gpu;
mod misc;
pub mod pipeline;

pub mod export {
    pub use {encase, nalgebra, wgpu};
}

// todo storage vs uniform buffers
