//! Buffers represent GPU memory allocations that can be bound to render and compute pipelines.

use crate::misc::ids::BufferId;

mod blas;
mod index;
pub mod mutability;
mod storage;
mod uniform;
mod vertex;

pub use blas::BlasBuffer;
pub use index::IndexBuffer;
pub use storage::StorageBuffer;
pub use uniform::UniformBuffer;
pub use vertex::VertexBuffer;

/// Represents a buffer that can be bound to a pipline.
pub trait BufferBinding {
    fn get_id(&self) -> BufferId;
}
