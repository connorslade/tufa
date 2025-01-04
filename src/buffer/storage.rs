use std::{marker::PhantomData, thread};

use anyhow::Result;
use encase::{
    internal::{CreateFrom, WriteInto},
    DynamicStorageBuffer, ShaderType,
};
use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingType, Buffer, BufferDescriptor, BufferUsages, MaintainBase, MapMode,
};

use crate::{
    gpu::Gpu,
    misc::{
        ids::BufferId,
        mutability::{Immutable, Mutability, Mutable},
        thread_ptr::ThreadSafePtr,
    },
};

use super::{Bindable, BindableResource};

/// A storage buffer is a buffer that can be read from or written to in the shader
pub struct StorageBuffer<T, Mut: Mutability> {
    gpu: Gpu,
    buffer: BufferId,

    _type: PhantomData<T>,
    _mut: PhantomData<Mut>,
}

impl<T: ShaderType + WriteInto + CreateFrom, Mut: Mutability> StorageBuffer<T, Mut> {
    fn get(&self) -> MappedRwLockReadGuard<Buffer> {
        RwLockReadGuard::map(self.gpu.buffers.read(), |x| &x[&self.buffer])
    }

    /// Uploads data into the buffer
    pub fn upload(&self, data: &T) -> Result<()> {
        let mut bytes = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut bytes);
        storage.write(&data)?;

        let buffer = self.get();
        if bytes.len() > buffer.size() as usize {
            drop(buffer);
            let replacement = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &bytes,
                usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            });
            self.gpu.buffers.write().insert(self.buffer, replacement);
        } else {
            self.gpu.queue.write_buffer(&buffer, 0, &bytes);
        }
        Ok(())
    }

    /// Downloads the buffer from the GPU in a blocking manner. This can be
    /// pretty slow.
    pub fn download(&self) -> Result<T> {
        let buffer = self.get();
        let staging = self.gpu.device.create_buffer(&BufferDescriptor {
            label: None,
            size: buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.gpu.dispatch(|encoder| {
            encoder.copy_buffer_to_buffer(&buffer, 0, &staging, 0, buffer.size());
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
        let buffer = self.get();
        let staging = self.gpu.device.create_buffer(&BufferDescriptor {
            label: None,
            size: buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let staging = Box::leak(Box::new(staging));

        self.gpu.dispatch(|encoder| {
            encoder.copy_buffer_to_buffer(&buffer, 0, staging, 0, buffer.size());
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

    let id = BufferId::new();
    let buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        contents: &buffer,
    });

    gpu.buffers.write().insert(id, buffer);

    Ok(StorageBuffer {
        gpu: gpu.clone(),
        buffer: id,

        _type: PhantomData,
        _mut: PhantomData,
    })
}

impl<T: ShaderType + WriteInto + CreateFrom> Bindable for StorageBuffer<T, Mutable> {
    fn resource(&self) -> BindableResource {
        BindableResource::Buffer(self.buffer)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}

impl<T: ShaderType + WriteInto + CreateFrom> Bindable for StorageBuffer<T, Immutable> {
    fn resource(&self) -> BindableResource {
        BindableResource::Buffer(self.buffer)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }
}

impl<T, Mut: Mutability> Drop for StorageBuffer<T, Mut> {
    fn drop(&mut self) {
        self.gpu.buffers.write().remove(&self.buffer);
    }
}
