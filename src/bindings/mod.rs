//! Items that can be bound to shaders. Includes textures, buffers of all kinds, and more!

use std::num::NonZeroU32;

use wgpu::{BindingType, Buffer, Sampler as WSampler, TextureView, TlasPackage};

use crate::misc::ids::{
    AccelerationStructureId, BufferId, SamplerId, TextureCollectionId, TextureId,
};

pub mod acceleration_structure;
pub mod buffer;
pub mod collection;
pub(crate) mod manager;
pub mod texture;

/// A resource that can be bound to a pipline.
pub trait Bindable {
    fn resource_id(&self) -> BindableResourceId;
    fn binding_type(&self) -> BindingType;
    fn count(&self) -> Option<NonZeroU32> {
        None
    }
}

/// The ID that maps to the [`BindableResource`] through the binding manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindableResourceId {
    Buffer(BufferId),
    Texture(TextureId),
    Sampler(SamplerId),
    AccelerationStructure(AccelerationStructureId),

    TextureCollection(TextureCollectionId),
}

/// Any resource that can be bound to a pipline.
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
