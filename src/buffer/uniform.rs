use std::marker::PhantomData;

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    ShaderType,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, Buffer, BufferUsages,
};

use crate::gpu::Gpu;

use super::Bindable;

pub struct UniformBuffer<T> {
    pub(crate) buffer: Buffer,
    pub(crate) _type: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + CreateFrom> UniformBuffer<T> {
    pub fn upload(&self, gpu: &Gpu, data: T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        gpu.queue.write_buffer(&self.buffer, 0, &buffer);
        Ok(())
    }
}

impl Gpu {
    pub fn create_uniform<T>(&mut self, data: T) -> Result<UniformBuffer<T>>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        let mut buffer = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            contents: &buffer,
        });

        Ok(UniformBuffer {
            buffer,
            _type: PhantomData,
        })
    }
}

impl<T> Bindable for UniformBuffer<T> {
    fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }
}
