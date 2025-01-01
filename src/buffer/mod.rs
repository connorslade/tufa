use wgpu::BindingResource;

mod storage;
mod uniform;

pub use storage::StorageBuffer;
pub use uniform::UniformBuffer;

/// A resource that can be bound to a shader
pub trait Bindable {
    fn as_entire_binding(&self) -> BindingResource<'_>;
}
