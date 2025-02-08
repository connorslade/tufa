use std::collections::HashMap;

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Device};

use crate::{misc::ids::PipelineId, pipeline::PipelineStatus};

use super::{BindableResource, BindableResourceId};

type RwMap<K, V> = RwLock<HashMap<K, V>>;

pub struct BindingManager {
    pipelines: RwMap<PipelineId, PipelineStatus>,
    resources: RwMap<BindableResourceId, BindableResource>,
}

impl BindingManager {
    pub fn new() -> Self {
        Self {
            pipelines: RwLock::new(HashMap::new()),
            resources: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) fn mark_resource_dirty(&self, resource: &BindableResourceId) {
        let mut pipelines = self.pipelines.write();
        for (_id, PipelineStatus { resources, dirty }) in pipelines.iter_mut() {
            *dirty |= resources.contains(resource);
        }
    }

    pub(crate) fn create_bind_group(
        &self,
        device: &Device,
        layout: &BindGroupLayout,
        entries: &[BindableResourceId],
    ) -> BindGroup {
        let resources = self.resources.read();

        let entries = &entries
            .iter()
            .enumerate()
            .map(|(binding, id)| BindGroupEntry {
                binding: binding as u32,
                resource: resources[id].as_binding(),
            })
            .collect::<Vec<_>>();

        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries,
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
    pub(crate) fn add_resource(
        &self,
        id: impl Into<BindableResourceId>,
        resource: impl Into<BindableResource>,
    ) {
        self.resources.write().insert(id.into(), resource.into());
    }

    pub(crate) fn get_resource(
        &self,
        id: impl Into<BindableResourceId>,
    ) -> MappedRwLockReadGuard<BindableResource> {
        RwLockReadGuard::map(self.resources.read(), |x| &x[&id.into()])
    }

    pub(crate) fn remove_resource(&self, id: impl Into<BindableResourceId>) {
        self.resources.write().remove(&id.into());
    }
}

impl Default for BindingManager {
    fn default() -> Self {
        Self::new()
    }
}
