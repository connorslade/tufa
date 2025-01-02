use wgpu::{BindingResource, BindingType};

mod index;
mod storage;
mod uniform;
mod vertex;

pub use index::IndexBuffer;
pub use storage::StorageBuffer;
pub use uniform::UniformBuffer;
pub use vertex::VertexBuffer;

/// A resource that can be bound to a shader
pub trait Bindable {
    fn as_entire_binding(&self) -> BindingResource<'_>;
    fn binding_type(&self) -> BindingType;
}
