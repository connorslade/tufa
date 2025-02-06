use std::collections::HashMap;

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, Device, TlasPackage,
};

use crate::{
    misc::ids::{AccelerationStructureId, BufferId, PipelineId},
    pipeline::PipelineStatus,
};

use super::BindableResource;

pub struct BindingManager {
    pub(crate) pipelines: RwLock<HashMap<PipelineId, PipelineStatus>>,
    pub(crate) buffers: RwLock<HashMap<BufferId, Buffer>>,
    pub(crate) acceleration_structures: RwLock<HashMap<AccelerationStructureId, TlasPackage>>,
}

impl BindingManager {
    pub fn new() -> Self {
        Self {
            pipelines: RwLock::new(HashMap::new()),
            buffers: RwLock::new(HashMap::new()),
            acceleration_structures: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) fn mark_resource_dirty(&self, resource: &BindableResource) {
        let mut pipelines = self.pipelines.write();
        for (_id, PipelineStatus { resources, dirty }) in pipelines.iter_mut() {
            *dirty |= resources.contains(resource);
        }
    }

    pub(crate) fn create_bind_group(
        &self,
        device: &Device,
        layout: &BindGroupLayout,
        entries: &[BindableResource],
    ) -> BindGroup {
        let buffers = self.buffers.read();
        let acceleration_structures = self.acceleration_structures.read();

        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries
                .iter()
                .enumerate()
                .map(|(binding, id)| BindGroupEntry {
                    binding: binding as u32,
                    resource: match id {
                        BindableResource::Buffer(id) => buffers[id].as_entire_binding(),
                        BindableResource::AccelerationStructure(id) => {
                            acceleration_structures[id].as_binding()
                        }
                    },
                })
                .collect::<Vec<_>>(),
        })
    }
}

impl BindingManager {
    pub(crate) fn add_pipeline(&self, id: PipelineId, status: PipelineStatus) {
        self.pipelines.write().insert(id, status);
    }

    pub(crate) fn get_pipeline(&self, id: PipelineId) -> MappedRwLockReadGuard<PipelineStatus> {
        RwLockReadGuard::map(self.pipelines.read(), |x| &x[&id])
    }

    pub(crate) fn remove_pipeline(&self, id: PipelineId) {
        self.pipelines.write().remove(&id);
    }
}

impl BindingManager {
    pub(crate) fn add_buffer(&self, id: BufferId, buffer: Buffer) {
        self.buffers.write().insert(id, buffer);
    }

    pub(crate) fn get_buffer(&self, id: BufferId) -> MappedRwLockReadGuard<Buffer> {
        RwLockReadGuard::map(self.buffers.read(), |x| &x[&id])
    }

    pub(crate) fn remove_buffer(&self, id: BufferId) {
        self.buffers.write().remove(&id);
    }
}

impl BindingManager {
    pub(crate) fn add_acceleration_structures(
        &self,
        id: AccelerationStructureId,
        package: TlasPackage,
    ) {
        self.acceleration_structures.write().insert(id, package);
    }

    pub(crate) fn remove_acceleration_structure(&self, id: AccelerationStructureId) {
        self.acceleration_structures.write().remove(&id);
    }
}
