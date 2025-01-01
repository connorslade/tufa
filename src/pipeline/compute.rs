use nalgebra::Vector3;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, ComputePassDescriptor};

use crate::{buffer::StorageBuffer, gpu::Gpu};

pub struct ComputePipelineBuilder<'a> {
    pub(crate) gpu: &'a mut Gpu,

    pub(crate) pipeline: wgpu::ComputePipeline,
    pub(crate) entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> ComputePipelineBuilder<'a> {
    pub fn bind_buffer<T>(mut self, entry: &'a StorageBuffer<T>) -> Self {
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
