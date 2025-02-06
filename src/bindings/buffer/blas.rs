use std::marker::PhantomData;

use anyhow::Result;
use encase::{internal::WriteInto, DynamicStorageBuffer, ShaderSize, ShaderType, StorageBuffer};
use parking_lot::MappedRwLockReadGuard;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingType, Buffer, BufferUsages,
};

use crate::{
    bindings::{Bindable, BindableResource},
    gpu::Gpu,
    misc::ids::BufferId,
};

pub struct BlasBuffer<T> {
    gpu: Gpu,
    buffer: BufferId,
    _type: PhantomData<T>,
}

impl<T> BlasBuffer<T> {
    pub(crate) fn get(&self) -> MappedRwLockReadGuard<Buffer> {
        self.gpu.binding_manager.get_buffer(self.buffer)
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
    pub fn create_blas<T>(&self, data: &[T]) -> Result<BlasBuffer<T>>
    where
        T: ShaderType + ShaderSize + WriteInto,
    {
        let mut buffer = Vec::new();
        let mut storage = DynamicStorageBuffer::new(&mut buffer);
        storage.write(data)?;

        let id = BufferId::new();
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE | BufferUsages::BLAS_INPUT,
            contents: &buffer,
        });

        self.binding_manager.add_buffer(id, buffer);
        Ok(BlasBuffer {
            gpu: self.clone(),
            buffer: id,
            _type: PhantomData,
        })
    }
}

impl<T> Bindable for BlasBuffer<T> {
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

impl<T> Drop for BlasBuffer<T> {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_buffer(self.buffer);
    }
}
