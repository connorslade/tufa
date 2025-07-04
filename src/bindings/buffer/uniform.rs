use std::marker::PhantomData;

use encase::{
    internal::{CreateFrom, WriteInto},
    ShaderType, StorageBuffer,
};
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

/// A uniform buffer is for passing small amounts of read-only data
pub struct UniformBuffer<T> {
    gpu: Gpu,
    buffer: BufferId,
    _type: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + CreateFrom> UniformBuffer<T> {
    fn get(&self) -> MappedRwLockReadGuard<'_, Buffer> {
        MappedRwLockReadGuard::map(self.gpu.binding_manager.get_resource(self.buffer), |x| {
            x.expect_buffer()
        })
    }

    /// Uploads data into the buffer
    pub fn upload(&self, data: &T) {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(data).unwrap();

        self.gpu.queue.write_buffer(&self.get(), 0, &buffer);
    }
}

impl Gpu {
    /// Creates a new uniform buffer with the given initial state
    pub fn create_uniform<T>(&self, data: &T) -> UniformBuffer<T>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(data).unwrap();

        let id = BufferId::new();
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            contents: &buffer,
        });

        self.binding_manager.add_resource(id, buffer);
        UniformBuffer {
            gpu: self.clone(),
            buffer: id,
            _type: PhantomData,
        }
    }
}

impl<T> BufferBinding for UniformBuffer<T> {
    fn get_id(&self) -> BufferId {
        self.buffer
    }
}

impl<T> Bindable for UniformBuffer<T> {
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::Buffer(self.buffer)
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
        self.gpu.binding_manager.remove_resource(self.buffer);
    }
}
