use std::{iter, marker::PhantomData, mem};

use anyhow::{Context, Result};
use encase::{
    internal::{CreateFrom, WriteInto},
    ShaderType, StorageBuffer,
};
use nalgebra::Vector3;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages, CommandEncoder,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, Device,
    DeviceDescriptor, Instance, InstanceDescriptor, MaintainBase, MapMode,
    PipelineCompilationOptions, PowerPreference, Queue, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource,
};

pub mod export {
    pub use {encase, nalgebra, wgpu};
}

pub struct Gpu {
    device: Device,
    queue: Queue,
}

impl Gpu {
    pub fn init() -> Result<Self> {
        let instance = Instance::new(InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .context("Error requesting adapter")?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&DeviceDescriptor::default(), None))?;

        Ok(Self { device, queue })
    }

    pub fn create_buffer<T>(&self) -> Buffer<T> {
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: None,
            size: 262144, 
            usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        Buffer {
            buffer,
            _type: PhantomData,
        }
    }

    pub fn create_buffer_init<T>(&mut self, data: T) -> Result<Buffer<T>>
    where
        T: ShaderType + WriteInto + CreateFrom,
    {
        let buffer = self.create_buffer::<T>();
        buffer.upload(self, data)?;
        Ok(buffer)
    }

    pub fn compute_pipeline(&mut self, source: ShaderSource) -> ComputePipelineBuilder {
        let module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source,
        });

        let pipeline = self
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &module,
                entry_point: Some("main"),
                // todo: pass in constants?
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            });

        ComputePipelineBuilder {
            gpu: self,
            pipeline,
            entries: Vec::new(),
        }
    }

    fn dispatch(&mut self, proc: impl FnOnce(&mut CommandEncoder)) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        proc(&mut encoder);
        self.queue.submit(iter::once(encoder.finish()));
    }
}

pub struct ComputePipelineBuilder<'a> {
    gpu: &'a mut Gpu,

    pipeline: wgpu::ComputePipeline,
    entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> ComputePipelineBuilder<'a> {
    pub fn bind_buffer<T>(mut self, entry: &'a Buffer<T>) -> Self {
        self.entries.push(BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: entry.buffer.as_entire_binding(),
        });
        self
    }

    pub fn finish(self) -> ComputePipeline {
        let bind_group = self.gpu.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &self.entries,
        });

        ComputePipeline {
            pipeline: self.pipeline,
            bind_group,
        }
    }
}

pub struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group: BindGroup,
}

impl ComputePipeline {
    pub fn dispatch(&self, gpu: &mut Gpu, workgroups: Vector3<u32>) {
        gpu.dispatch(|encoder| {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, Some(&self.bind_group), &[]);
            compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
        });
    }
}

// todo storage vs uniform buffers

pub struct Buffer<T> {
    buffer: wgpu::Buffer,
    _type: PhantomData<T>,
}

impl<T: ShaderType + WriteInto + CreateFrom> Buffer<T> {
    pub fn upload(&self, gpu: &mut Gpu, data: T) -> Result<()> {
        let mut buffer = Vec::new();
        let mut storage = StorageBuffer::new(&mut buffer);
        storage.write(&data)?;

        gpu.queue.write_buffer(&self.buffer, 0, &buffer);
        Ok(())
    }

    pub fn download(&self, gpu: &mut Gpu) -> Result<T> {
        let staging = gpu.device.create_buffer(&BufferDescriptor {
            label: None,
            size: 262144,
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
