use wgpu::BindingType;

use crate::misc::ids::{AccelerationStructureId, BufferId};

pub mod acceleration_structure;
mod buffer;
pub mod manager;
pub use buffer::blas::BlasBuffer;
pub use buffer::index::IndexBuffer;
pub use buffer::storage::StorageBuffer;
pub use buffer::uniform::UniformBuffer;
pub use buffer::vertex::VertexBuffer;

/// A resource that can be bound to a shader
pub trait Bindable {
    fn resource(&self) -> BindableResource;
    fn binding_type(&self) -> BindingType;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BindableResource {
    Buffer(BufferId),
    AccelerationStructure(AccelerationStructureId),
}
