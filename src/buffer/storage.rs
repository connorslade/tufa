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

use crate::{
    gpu::Gpu,
    misc::{
        mutability::{Immutable, Mutability, Mutable},
        thread_ptr::ThreadSafePtr,
    },
};

use super::Bindable;

/// A storage buffer is a buffer that can be read from or written to in the shader
pub struct StorageBuffer<T, Mut: Mutability> {
    gpu: Gpu,
    buffer: Buffer,

    _type: PhantomData<T>,
    _mut: PhantomData<Mut>,
}

impl<T: ShaderType + WriteInto + CreateFrom, Mut: Mutability> StorageBuffer<T, Mut> {
    /// Uploads data into the buffer
    pub fn upload(&mut self, data: &T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        if buffer.len() > self.buffer.size() as usize {
            self.buffer = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &buffer,
                usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            });
        } else {
            self.gpu.queue.write_buffer(&self.buffer, 0, &buffer);
        }
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
    pub fn create_storage<T>(&self, data: T) -> Result<StorageBuffer<T, Mutable>>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        create_storage(self, data)
    }

    pub fn create_storage_read<T>(&self, data: T) -> Result<StorageBuffer<T, Immutable>>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        create_storage(self, data)
    }
}

fn create_storage<T, Mut: Mutability>(gpu: &Gpu, data: T) -> Result<StorageBuffer<T, Mut>>
where
    T: ShaderType + WriteInto + CreateFrom,
{
    let mut buffer = Vec::new();
    let mut storage = encase::StorageBuffer::new(&mut buffer);
    storage.write(&data)?;

    let buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        contents: &buffer,
    });

    Ok(StorageBuffer {
        gpu: gpu.clone(),
        buffer,

        _type: PhantomData,
        _mut: PhantomData,
    })
}

impl<T> Bindable for StorageBuffer<T, Mutable> {
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

impl<T> Bindable for StorageBuffer<T, Immutable> {
    fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}
