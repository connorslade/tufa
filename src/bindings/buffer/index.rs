use anyhow::Result;
use parking_lot::MappedRwLockReadGuard;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingType, Buffer, BufferUsages,
};

use crate::{
    bindings::{Bindable, BindableResourceId},
    gpu::Gpu,
    misc::ids::BufferId,
};

pub struct IndexBuffer {
    gpu: Gpu,
    buffer: BufferId,
}

impl IndexBuffer {
    pub(crate) fn get(&self) -> MappedRwLockReadGuard<Buffer> {
        MappedRwLockReadGuard::map(self.gpu.binding_manager.get_resource(self.buffer), |x| {
            x.expect_buffer()
        })
    }

    pub fn upload(&self, data: &[u32]) -> Result<()> {
        let buffer = bytemuck::cast_slice(data);
        self.gpu.queue.write_buffer(&self.get(), 0, buffer);
        Ok(())
    }
}

impl Gpu {
    pub fn create_index(&self, data: &[u32]) -> Result<IndexBuffer> {
        let id = BufferId::new();
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            contents: bytemuck::cast_slice(data),
        });

        self.binding_manager.add_resource(id, buffer);
        Ok(IndexBuffer {
            gpu: self.clone(),
            buffer: id,
        })
    }
}

impl Bindable for IndexBuffer {
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::Buffer(self.buffer)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_resource(self.buffer);
    }
}
