use wgpu::BindingType;

mod index;
mod storage;
mod uniform;
mod vertex;

pub use index::IndexBuffer;
pub use storage::StorageBuffer;
pub use uniform::UniformBuffer;
pub use vertex::VertexBuffer;

use crate::misc::ids::BufferId;

/// A resource that can be bound to a shader
pub trait Bindable {
    fn resource(&self) -> BindableResource;
    fn binding_type(&self) -> BindingType;
}

pub enum BindableResource {
    Buffer(BufferId),
}
