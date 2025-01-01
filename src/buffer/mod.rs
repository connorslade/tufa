use wgpu::BindingResource;

mod storage;
mod uniform;

pub use storage::StorageBuffer;
pub use uniform::UniformBuffer;

pub trait Bindable {
    fn as_entire_binding(&self) -> BindingResource<'_>;
}
