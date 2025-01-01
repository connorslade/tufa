use std::{marker::PhantomData, mem};

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    ShaderType,
};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, MaintainBase, MapMode};

use crate::gpu::Gpu;

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

    pub fn download(&self, gpu: &mut Gpu) -> Result<T> {
        let staging = gpu.device.create_buffer(&BufferDescriptor {
            label: None,
            size: mem::size_of::<T>() as u64,
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
