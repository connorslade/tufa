use std::marker::PhantomData;

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    DynamicStorageBuffer, ShaderSize, ShaderType, StorageBuffer,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, BindingType, Buffer, BufferUsages,
};

use crate::gpu::Gpu;

use super::Bindable;

pub struct VertexBuffer<T> {
    gpu: Gpu,
    pub(crate) buffer: Buffer,
    _type: PhantomData<T>,
}

impl<T> VertexBuffer<T> {
    pub fn upload(&self, data: &[T]) -> Result<()>
    where
        T: ShaderType + ShaderSize + WriteInto,
    {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        self.gpu.queue.write_buffer(&self.buffer, 0, &buffer);
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

        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            contents: &buffer,
        });

        Ok(VertexBuffer {
            gpu: self.clone(),
            buffer,
            _type: PhantomData,
        })
    }
}

impl<T> Bindable for VertexBuffer<T> {
    fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}
