use std::marker::PhantomData;

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    ShaderType, StorageBuffer,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, BindingType, Buffer, BufferUsages,
};

use crate::gpu::Gpu;

use super::Bindable;

/// A uniform buffer is for passing small amounts of read-only data
pub struct UniformBuffer<T> {
    gpu: Gpu,
    buffer: Buffer,
    _type: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + CreateFrom> UniformBuffer<T> {
    /// Uploads data into the buffer
    pub fn upload(&self, data: &T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(data)?;

        self.gpu.queue.write_buffer(&self.buffer, 0, &buffer);
        Ok(())
    }
}

impl Gpu {
    /// Creates a new uniform buffer with the givin initial state
    pub fn create_uniform<T>(&self, data: T) -> Result<UniformBuffer<T>>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            contents: &buffer,
        });

        Ok(UniformBuffer {
            gpu: self.clone(),
            buffer,
            _type: PhantomData,
        })
    }
}

impl<T> Bindable for UniformBuffer<T> {
    fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}
