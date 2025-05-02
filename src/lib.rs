#![doc = include_str!("../README.md")]

use wgpu::TextureFormat;

pub mod bindings;
pub mod gpu;
#[cfg(feature = "interactive")]
pub mod interactive;
pub mod misc;
pub mod pipeline;

pub mod export {
    //! Exported types from crates tufa uses internally.

    #[cfg(feature = "interactive")]
    pub use {egui, winit};
    pub use {encase, nalgebra, wgpu};
}

pub mod prelude {
    //! Common imports from tufa.
    //!
    //! See [Module prelude](https://doc.rust-lang.org/std/prelude/index.html).

    pub use crate::{
        bindings::{buffer::*, texture::*},
        export::wgpu::{include_wgsl, RenderPass, ShaderStages},
        gpu::Gpu,
        pipeline::render::{RenderPipeline, Vertex},
    };

    #[cfg(feature = "interactive")]
    pub use crate::{
        export::{
            egui::Context,
            winit::{
                event::{DeviceEvent, DeviceId, WindowEvent},
                window::WindowAttributes,
            },
        },
        interactive::{GraphicsCtx, Interactive},
    };
}

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth24PlusStencil8;
