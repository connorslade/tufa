use wgpu::BindingType;

pub mod manager;
mod resources;
pub use resources::*;

use crate::misc::ids::{AccelerationStructureId, BufferId};

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
