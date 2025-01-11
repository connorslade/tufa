use std::marker::PhantomData;

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    ShaderType, StorageBuffer,
};
use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingType, Buffer, BufferUsages,
};

use crate::{gpu::Gpu, misc::ids::BufferId};

use super::{Bindable, BindableResource};

/// A uniform buffer is for passing small amounts of read-only data
pub struct UniformBuffer<T> {
    gpu: Gpu,
    buffer: BufferId,
    _type: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + CreateFrom> UniformBuffer<T> {
    fn get(&self) -> MappedRwLockReadGuard<Buffer> {
        RwLockReadGuard::map(self.gpu.buffers.read(), |x| &x[&self.buffer])
    }

    /// Uploads data into the buffer
    pub fn upload(&self, data: &T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(data)?;

        self.gpu.queue.write_buffer(&self.get(), 0, &buffer);
        Ok(())
    }
}

impl Gpu {
    /// Creates a new uniform buffer with the givin initial state
    pub fn create_uniform<T>(&self, data: &T) -> Result<UniformBuffer<T>>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(data)?;

        let id = BufferId::new();
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            contents: &buffer,
        });

        self.buffers.write().insert(id, buffer);

        Ok(UniformBuffer {
            gpu: self.clone(),
            buffer: id,
            _type: PhantomData,
        })
    }
}

impl<T> Bindable for UniformBuffer<T> {
    fn resource(&self) -> BindableResource {
        BindableResource::Buffer(self.buffer)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}

impl<T> Drop for UniformBuffer<T> {
    fn drop(&mut self) {
        self.gpu.buffers.write().remove(&self.buffer);
    }
}
