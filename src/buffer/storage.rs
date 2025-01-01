use std::marker::PhantomData;

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    ShaderType,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, Buffer, BufferDescriptor, BufferUsages, MaintainBase, MapMode,
};

use crate::gpu::Gpu;

use super::Bindable;

pub struct StorageBuffer<T> {
    pub(crate) buffer: Buffer,
    pub(crate) _type: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + CreateFrom> StorageBuffer<T> {
    pub fn upload(&self, gpu: &mut Gpu, data: T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        gpu.queue.write_buffer(&self.buffer, 0, &buffer);
        Ok(())
    }

    pub fn download(&self, gpu: &Gpu) -> Result<T> {
        let staging = gpu.device.create_buffer(&BufferDescriptor {
            label: None,
            size: self.buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        gpu.dispatch(|encoder| {
            encoder.copy_buffer_to_buffer(&self.buffer, 0, &staging, 0, self.buffer.size());
        });

        let slice = staging.slice(..);

        let (tx, rx) = crossbeam_channel::bounded(1);
        slice.map_async(MapMode::Read, move |_| tx.send(()).unwrap());

        gpu.device.poll(MaintainBase::Wait);
        rx.recv().unwrap();

        let data = slice.get_mapped_range().to_vec();
        let mut store = encase::DynamicStorageBuffer::new(data);

        Ok(store.create()?)
    }
}

impl Gpu {
    pub fn create_storage<T>(&mut self, data: T) -> Result<StorageBuffer<T>>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        let mut buffer = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            contents: &buffer,
        });

        Ok(StorageBuffer {
            buffer,
            _type: PhantomData,
        })
    }
}

impl<T> Bindable for StorageBuffer<T> {
    fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }
}
