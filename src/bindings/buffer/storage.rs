use std::{marker::PhantomData, thread};

use encase::{
    internal::{CreateFrom, WriteInto},
    DynamicStorageBuffer, ShaderType,
};
use parking_lot::MappedRwLockReadGuard;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingType, Buffer, BufferDescriptor, BufferUsages, MaintainBase, MapMode,
};

use crate::{
    bindings::{
        buffer::mutability::{Immutable, Mutability, Mutable},
        Bindable, BindableResourceId,
    },
    gpu::Gpu,
    misc::{ids::BufferId, thread_ptr::ThreadSafePtr},
};

use super::BufferBinding;

/// A storage buffer is a buffer that can be read from or written to in the shader
pub struct StorageBuffer<T, Mut: Mutability> {
    pub(crate) gpu: Gpu,
    pub(crate) buffer: BufferId,

    _type: PhantomData<T>,
    _mut: PhantomData<Mut>,
}

impl<T: ShaderType + WriteInto + CreateFrom, Mut: Mutability> StorageBuffer<T, Mut> {
    fn get(&self) -> MappedRwLockReadGuard<'_, Buffer> {
        MappedRwLockReadGuard::map(self.gpu.binding_manager.get_resource(self.buffer), |x| {
            x.expect_buffer()
        })
    }

    /// Uploads data into the buffer
    pub fn upload(&self, data: &T) {
        self.upload_inner(data, false)
    }

    /// Uploads data into the buffer, reallocating to the minimum needed buffer size.
    pub fn upload_shrink(&self, data: &T) {
        self.upload_inner(data, true)
    }

    fn upload_inner(&self, data: &T, shrink: bool) {
        let mut bytes = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut bytes);
        storage.write(data).unwrap();

        let buffer = self.get();
        let current_size = buffer.size() as usize;

        if (bytes.len() > current_size) || (bytes.len() != current_size && shrink) {
            drop(buffer);
            let replacement = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &bytes,
                usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            });

            let binding_manager = &self.gpu.binding_manager;
            binding_manager.add_resource(self.buffer, replacement);
            binding_manager.mark_resource_dirty(&BindableResourceId::Buffer(self.buffer));
        } else {
            self.gpu.queue.write_buffer(&buffer, 0, &bytes);
        }
    }

    /// Downloads the buffer from the GPU in a blocking manner. This can be
    /// pretty slow.
    pub fn download(&self) -> T {
        let buffer = self.get();
        let staging = self.gpu.device.create_buffer(&BufferDescriptor {
            label: None,
            size: buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.gpu.immediate_dispatch(|encoder| {
            encoder.copy_buffer_to_buffer(&buffer, 0, &staging, 0, buffer.size());
        });

        let slice = staging.slice(..);

        let (tx, rx) = crossbeam_channel::bounded(1);
        slice.map_async(MapMode::Read, move |_| tx.send(()).unwrap());

        self.gpu.device.poll(MaintainBase::Wait);
        rx.recv().unwrap();

        let data = slice.get_mapped_range().to_vec();
        let mut store = encase::DynamicStorageBuffer::new(data);

        store.create().unwrap()
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

        self.gpu.immediate_dispatch(|encoder| {
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
    pub fn create_storage<T, Mut: Mutability>(&self, data: &T) -> StorageBuffer<T, Mut>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        let mut buffer = Vec::new();
        let mut storage = encase::StorageBuffer::new(&mut buffer);
        storage.write(data).unwrap();

        let id = BufferId::new();
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            contents: &buffer,
        });

        self.binding_manager.add_resource(id, buffer);
        StorageBuffer {
            gpu: self.clone(),
            buffer: id,

            _type: PhantomData,
            _mut: PhantomData,
        }
    }

    pub fn create_storage_empty<T, Mut: Mutability>(&self, size: u64) -> StorageBuffer<T, Mut>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        let id = BufferId::new();
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            mapped_at_creation: false,
            size,
        });

        self.binding_manager.add_resource(id, buffer);
        StorageBuffer {
            gpu: self.clone(),
            buffer: id,

            _type: PhantomData,
            _mut: PhantomData,
        }
    }
}

impl<T, Mut: Mutability> BufferBinding for StorageBuffer<T, Mut> {
    fn get_id(&self) -> BufferId {
        self.buffer
    }
}

impl<T: ShaderType + WriteInto + CreateFrom> Bindable for StorageBuffer<T, Mutable> {
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::Buffer(self.buffer)
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
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::Buffer(self.buffer)
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
        self.gpu.binding_manager.remove_resource(self.buffer);
    }
}
