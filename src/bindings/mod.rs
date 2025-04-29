use std::num::NonZeroU32;

use wgpu::{BindingType, Buffer, Sampler as WSampler, TextureView, TlasPackage};

use crate::misc::ids::{
    AccelerationStructureId, BufferId, SamplerId, TextureCollectionId, TextureId,
};

pub mod acceleration_structure;
mod buffer;
mod collection;
pub mod manager;
mod sampler;
pub mod texture;

pub use buffer::blas::BlasBuffer;
pub use buffer::index::IndexBuffer;
pub use buffer::mutability;
pub use buffer::storage::StorageBuffer;
pub use buffer::uniform::UniformBuffer;
pub use buffer::vertex::VertexBuffer;
pub use collection::texture_collection::TextureCollection;
pub use sampler::Sampler;
pub use texture::Texture;

/// A resource that can be bound to a shader
pub trait Bindable {
    fn resource_id(&self) -> BindableResourceId;
    fn binding_type(&self) -> BindingType;
    fn count(&self) -> Option<NonZeroU32> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindableResourceId {
    Buffer(BufferId),
    Texture(TextureId),
    Sampler(SamplerId),
    AccelerationStructure(AccelerationStructureId),

    TextureCollection(TextureCollectionId),
}

pub enum BindableResource {
    Buffer(Buffer),
    Texture(TextureView),
    Sampler(WSampler),
    AccelerationStructure(TlasPackage),
}

impl BindableResource {
    pub fn expect_buffer(&self) -> &Buffer {
        match self {
            BindableResource::Buffer(buffer) => buffer,
            _ => panic!("Expected buffer"),
        }
    }

    pub fn expect_texture_view(&self) -> &TextureView {
        match self {
            BindableResource::Texture(texture_view) => texture_view,
            _ => panic!("Expected texture view"),
        }
    }

    pub fn expect_tlas_package(&self) -> &TlasPackage {
        match self {
            BindableResource::AccelerationStructure(tlas_package) => tlas_package,
            _ => panic!("Expected TLAS package"),
        }
    }
}

impl From<Buffer> for BindableResource {
    fn from(val: Buffer) -> Self {
        BindableResource::Buffer(val)
    }
}

impl From<TextureView> for BindableResource {
    fn from(val: TextureView) -> Self {
        BindableResource::Texture(val)
    }
}

impl From<WSampler> for BindableResource {
    fn from(val: WSampler) -> Self {
        BindableResource::Sampler(val)
    }
}

impl From<TlasPackage> for BindableResource {
    fn from(val: TlasPackage) -> Self {
        BindableResource::AccelerationStructure(val)
    }
}
