use std::collections::HashMap;

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
};

use crate::{
    misc::ids::{PipelineId, TextureCollectionId, TextureId},
    pipeline::PipelineStatus,
};

use super::{BindableResource, BindableResourceId};

type RwMap<K, V> = RwLock<HashMap<K, V>>;

// todo: reference count resources
pub struct BindingManager {
    pipelines: RwMap<PipelineId, PipelineStatus>,
    resources: RwMap<BindableResourceId, BindableResource>,
    collections: RwMap<TextureCollectionId, Vec<TextureId>>,
}

impl BindingManager {
    pub fn new() -> Self {
        Self {
            pipelines: RwLock::new(HashMap::new()),
            resources: RwLock::new(HashMap::new()),
            collections: RwLock::new(HashMap::new()),
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
        let collections = self.collections.read();

        let mut collection_id = 0;
        let collections = entries
            .iter()
            .filter_map(|x| match x {
                BindableResourceId::TextureCollection(id) => {
                    let collection = collections[id]
                        .iter()
                        .map(|&x| resources[&x.into()].expect_texture_view())
                        .collect::<Vec<_>>();
                    Some(collection)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let entries = &entries
            .iter()
            .enumerate()
            .map(|(binding, id)| BindGroupEntry {
                binding: binding as u32,
                resource: match id {
                    BindableResourceId::TextureCollection(_) => {
                        collection_id += 1;
                        // BindingResource::TextureViewArray(&)
                        BindingResource::TextureView(&collections[collection_id - 1][0])
                    }
                    x => match &resources[x] {
                        BindableResource::Buffer(buffer) => buffer.as_entire_binding(),
                        BindableResource::Texture(texture_view) => {
                            BindingResource::TextureView(texture_view)
                        }
                        BindableResource::Sampler(sampler) => BindingResource::Sampler(sampler),
                        BindableResource::AccelerationStructure(tlas_package) => {
                            tlas_package.as_binding()
                        }
                    },
                },
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

    pub(crate) fn add_collection(&self, id: TextureCollectionId, resources: Vec<TextureId>) {
        self.collections.write().insert(id, resources);
    }

    pub(crate) fn get_collection(
        &self,
        id: TextureCollectionId,
    ) -> MappedRwLockReadGuard<Vec<TextureId>> {
        RwLockReadGuard::map(self.collections.read(), |x| &x[&id])
    }

    pub(crate) fn renove_collection(&self, id: TextureCollectionId) {
        self.collections.write().remove(&id);
    }
}

impl Default for BindingManager {
    fn default() -> Self {
        Self::new()
    }
}
