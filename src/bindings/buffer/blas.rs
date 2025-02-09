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

pub struct BlasBuffer<T> {
    gpu: Gpu,
    buffer: BufferId,
    _type: PhantomData<T>,
}

impl<T> BlasBuffer<T> {
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

        self.binding_manager.add_resource(id, buffer);
        Ok(BlasBuffer {
            gpu: self.clone(),
            buffer: id,
            _type: PhantomData,
        })
    }
}

impl<T> Bindable for BlasBuffer<T> {
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::Buffer(self.buffer)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}

impl<T> Clone for BlasBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            gpu: self.gpu.clone(),
            buffer: self.buffer,
            _type: PhantomData,
        }
    }
}

impl<T> Drop for BlasBuffer<T> {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_resource(self.buffer);
    }
}
