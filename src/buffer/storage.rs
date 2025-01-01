use std::{marker::PhantomData, thread};

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    DynamicStorageBuffer, ShaderType,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, Buffer, BufferDescriptor, BufferUsages, MaintainBase, MapMode,
};

use crate::{gpu::Gpu, misc::ThreadSafePtr};

use super::Bindable;

pub struct StorageBuffer<T> {
    pub(crate) gpu: Gpu,
    pub(crate) buffer: Buffer,
    pub(crate) _type: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + CreateFrom> StorageBuffer<T> {
    pub fn upload(&self, data: T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        self.gpu.queue.write_buffer(&self.buffer, 0, &buffer);
        Ok(())
    }

    pub fn download(&self) -> Result<T> {
        let staging = self.gpu.device.create_buffer(&BufferDescriptor {
            label: None,
            size: self.buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.gpu.dispatch(|encoder| {
            encoder.copy_buffer_to_buffer(&self.buffer, 0, &staging, 0, self.buffer.size());
        });

        let slice = staging.slice(..);

        let (tx, rx) = crossbeam_channel::bounded(1);
        slice.map_async(MapMode::Read, move |_| tx.send(()).unwrap());

        self.gpu.device.poll(MaintainBase::Wait);
        rx.recv().unwrap();

        let data = slice.get_mapped_range().to_vec();
        let mut store = encase::DynamicStorageBuffer::new(data);

        Ok(store.create()?)
    }

    pub fn download_async(&self, func: impl FnOnce(T) + Send + 'static) {
        let staging = self.gpu.device.create_buffer(&BufferDescriptor {
            label: None,
            size: self.buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let staging = Box::leak(Box::new(staging));

        self.gpu.dispatch(|encoder| {
            encoder.copy_buffer_to_buffer(&self.buffer, 0, staging, 0, self.buffer.size());
        });

        let staging = ThreadSafePtr(staging as *mut Buffer);
        let slice = staging.deref().slice(..);

        slice.map_async(MapMode::Read, move |_| {
            let data = slice.get_mapped_range().to_vec();
            unsafe { drop(Box::from_raw(staging.deref_mut())) };

            thread::spawn(move || {
                let mut store = DynamicStorageBuffer::new(data);
                func(store.create().unwrap());
            });
        });
    }
}

impl Gpu {
    pub fn create_storage<T>(&self, data: T) -> Result<StorageBuffer<T>>
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
            gpu: self.clone(),
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
