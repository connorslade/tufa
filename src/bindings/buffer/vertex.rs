use std::marker::PhantomData;

use anyhow::Result;
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

    pub fn upload(&self, data: &[T]) -> Result<()>
    where
        T: ShaderType + ShaderSize + WriteInto,
    {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        self.gpu.queue.write_buffer(&self.get(), 0, &buffer);
        Ok(())
    }
}

impl Gpu {
    pub fn create_vertex<T>(&self, data: &[T]) -> Result<VertexBuffer<T>>
    where
        T: ShaderType + ShaderSize + WriteInto,
    {
        let mut buffer = Vec::new();
        let mut storage = DynamicStorageBuffer::new(&mut buffer);
        storage.write(data)?;

        let id = BufferId::new();
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            contents: &buffer,
        });

        self.binding_manager.add_resource(id, buffer);
        Ok(VertexBuffer {
            gpu: self.clone(),
            buffer: id,
            _type: PhantomData,
        })
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
