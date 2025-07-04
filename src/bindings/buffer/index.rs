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

use super::BufferBinding;

/// Represents the indices of a mesh for rendering.
pub struct IndexBuffer {
    gpu: Gpu,
    buffer: BufferId,
}

impl IndexBuffer {
    pub(crate) fn get(&self) -> MappedRwLockReadGuard<'_, Buffer> {
        MappedRwLockReadGuard::map(self.gpu.binding_manager.get_resource(self.buffer), |x| {
            x.expect_buffer()
        })
    }

    pub fn upload(&self, data: &[u32]) {
        let buffer = bytemuck::cast_slice(data);

        let this = self.get();
        if buffer.len() as u64 > this.size() {
            drop(this);
            let replacement = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: buffer,
                usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            });

            let binding_manager = &self.gpu.binding_manager;
            binding_manager.add_resource(self.buffer, replacement);
            binding_manager.mark_resource_dirty(&BindableResourceId::Buffer(self.buffer));
        } else {
            self.gpu.queue.write_buffer(&this, 0, buffer);
        }
    }
}

impl Gpu {
    pub fn create_index(&self, data: &[u32]) -> IndexBuffer {
        let id = BufferId::new();
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            contents: bytemuck::cast_slice(data),
        });

        self.binding_manager.add_resource(id, buffer);
        IndexBuffer {
            gpu: self.clone(),
            buffer: id,
        }
    }

    pub fn create_index_empty(&self, size: usize) -> IndexBuffer {
        let id = BufferId::new();
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (size * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            mapped_at_creation: false,
        });

        self.binding_manager.add_resource(id, buffer);
        IndexBuffer {
            gpu: self.clone(),
            buffer: id,
        }
    }
}

impl BufferBinding for IndexBuffer {
    fn get_id(&self) -> BufferId {
        self.buffer
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
