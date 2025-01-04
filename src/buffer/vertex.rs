use std::marker::PhantomData;

use anyhow::Result;
use encase::{internal::WriteInto, DynamicStorageBuffer, ShaderSize, ShaderType, StorageBuffer};
use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingType, Buffer, BufferUsages,
};

use crate::{gpu::Gpu, misc::ids::BufferId};

use super::{Bindable, BindableResource};

pub struct VertexBuffer<T> {
    gpu: Gpu,
    buffer: BufferId,
    _type: PhantomData<T>,
}

impl<T> VertexBuffer<T> {
    pub(crate) fn get(&self) -> MappedRwLockReadGuard<Buffer> {
        RwLockReadGuard::map(self.gpu.buffers.read(), |x| &x[&self.buffer])
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

        self.buffers.write().insert(id, buffer);

        Ok(VertexBuffer {
            gpu: self.clone(),
            buffer: id,
            _type: PhantomData,
        })
    }
}

impl<T> Bindable for VertexBuffer<T> {
    fn resource(&self) -> BindableResource {
        BindableResource::Buffer(self.buffer)
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
        self.gpu.buffers.write().remove(&self.buffer);
    }
}
