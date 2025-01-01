use std::{iter, marker::PhantomData, mem};

use anyhow::{Context, Result};
use encase::{
    internal::{CreateFrom, WriteInto},
    ShaderType,
};
use wgpu::{
    BufferDescriptor, BufferUsages, CommandEncoder, CommandEncoderDescriptor,
    ComputePipelineDescriptor, Device, DeviceDescriptor, Instance, InstanceDescriptor,
    PipelineCompilationOptions, PowerPreference, Queue, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource,
};

use crate::{buffer::StorageBuffer, pipeline::compute::ComputePipelineBuilder};

pub struct Gpu {
    pub(crate) device: Device,
    pub(crate) queue: Queue,
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

    pub fn create_buffer<T>(&self) -> StorageBuffer<T> {
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: None,
            size: mem::size_of::<T>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        StorageBuffer {
            buffer,
            _type: PhantomData,
        }
    }

    pub fn create_buffer_init<T>(&mut self, data: T) -> Result<StorageBuffer<T>>
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

    pub(crate) fn dispatch(&mut self, proc: impl FnOnce(&mut CommandEncoder)) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        proc(&mut encoder);
        self.queue.submit(iter::once(encoder.finish()));
    }
}
