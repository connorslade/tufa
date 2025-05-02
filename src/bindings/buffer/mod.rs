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

pub trait BufferBinding {
    fn get_id(&self) -> BufferId;
}
