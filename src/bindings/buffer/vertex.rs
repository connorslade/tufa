use std::marker::PhantomData;

use encase::{internal::WriteInto, DynamicStorageBuffer, ShaderSize, ShaderType, StorageBuffer};
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

/// Represents the vertices of a mesh for rendering.
pub struct VertexBuffer<T> {
    gpu: Gpu,
    buffer: BufferId,
    _type: PhantomData<T>,
}

impl<T> VertexBuffer<T> {
    pub(crate) fn get(&self) -> MappedRwLockReadGuard<Buffer> {
        MappedRwLockReadGuard::map(self.gpu.binding_manager.get_resource(self.buffer), |x| {
            x.expect_buffer()
        })
    }

    pub fn upload(&self, data: &[T])
    where
        T: ShaderType + ShaderSize + WriteInto,
    {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(&data).unwrap();

        let this = self.get();
        if buffer.len() as u64 > this.size() {
            drop(this);
            let replacement = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &buffer,
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            });

            let binding_manager = &self.gpu.binding_manager;
            binding_manager.add_resource(self.buffer, replacement);
            binding_manager.mark_resource_dirty(&BindableResourceId::Buffer(self.buffer));
        } else {
            self.gpu.queue.write_buffer(&this, 0, &buffer);
        }
    }
}

impl Gpu {
    pub fn create_vertex<T>(&self, data: &[T]) -> VertexBuffer<T>
    where
        T: ShaderType + ShaderSize + WriteInto,
    {
        let mut buffer = Vec::new();
        let mut storage = DynamicStorageBuffer::new(&mut buffer);
        storage.write(data).unwrap();

        let id = BufferId::new();
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            contents: &buffer,
        });

        self.binding_manager.add_resource(id, buffer);
        VertexBuffer {
            gpu: self.clone(),
            buffer: id,
            _type: PhantomData,
        }
    }

    pub fn create_vertex_empty<T>(&self, size: usize) -> VertexBuffer<T>
    where
        T: ShaderType + ShaderSize + WriteInto,
    {
        let id = BufferId::new();
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<T>() * size) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        self.binding_manager.add_resource(id, buffer);
        VertexBuffer {
            gpu: self.clone(),
            buffer: id,
            _type: PhantomData,
        }
    }
}

impl<T> BufferBinding for VertexBuffer<T> {
    fn get_id(&self) -> BufferId {
        self.buffer
    }
}

impl<T> Bindable for VertexBuffer<T> {
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

impl<T> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_resource(self.buffer);
    }
}
