use anyhow::Result;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, BindingType, Buffer, BufferUsages,
};

use crate::gpu::Gpu;

use super::Bindable;

pub struct IndexBuffer {
    gpu: Gpu,
    pub(crate) buffer: Buffer,
}

impl IndexBuffer {
    pub fn upload(&self, data: &[u32]) -> Result<()> {
        let buffer = bytemuck::cast_slice(data);
        self.gpu.queue.write_buffer(&self.buffer, 0, buffer);
        Ok(())
    }
}

impl Gpu {
    pub fn create_index(&self, data: &[u32]) -> Result<IndexBuffer> {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            contents: bytemuck::cast_slice(data),
        });

        Ok(IndexBuffer {
            gpu: self.clone(),
            buffer,
        })
    }
}

impl Bindable for IndexBuffer {
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
