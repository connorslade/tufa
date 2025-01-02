use std::{marker::PhantomData, thread};

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    DynamicStorageBuffer, ShaderType,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, BindingType, Buffer, BufferDescriptor, BufferUsages, MaintainBase, MapMode,
};

use crate::{gpu::Gpu, misc::thread_ptr::ThreadSafePtr};

use super::Bindable;

/// A storage buffer is a buffer that can be read from or written to in the shader
pub struct StorageBuffer<T> {
    gpu: Gpu,
    buffer: Buffer,
    _type: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + CreateFrom> StorageBuffer<T> {
    /// Uploads data into the buffer
    pub fn upload(&self, data: T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        self.gpu.queue.write_buffer(&self.buffer, 0, &buffer);
        Ok(())
    }

    /// Downloads the buffer from the GPU in a blocking manner. This can be
    /// pretty slow.
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

    /// Requests the download of the buffer. The provided callback will be
    /// executed once the transfer finishes.
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
    /// Creates a new storage buffer with the givin initial state
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

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}
