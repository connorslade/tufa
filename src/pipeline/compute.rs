use nalgebra::Vector3;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, ComputePassDescriptor,
    ComputePipelineDescriptor, PipelineCompilationOptions, ShaderModuleDescriptor,
};

use crate::{buffer::Bindable, gpu::Gpu};

pub struct ComputePipelineBuilder<'a> {
    pub(crate) gpu: Gpu,

    pub(crate) pipeline: wgpu::ComputePipeline,
    pub(crate) entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> ComputePipelineBuilder<'a> {
    /// Adds the supplied buffer as the next entry in the bind group, starting with binding zero and counting up.
    pub fn bind_buffer(mut self, entry: &'a impl Bindable) -> Self {
        self.entries.push(BindGroupEntry {
            binding: self.entries.len() as u32,
            resource: entry.as_entire_binding(),
        });
        self
    }

    /// Converts the pipeline builder into an actual compte pipeline
    pub fn finish(self) -> ComputePipeline {
        let bind_group = self.gpu.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &self.entries,
        });

        ComputePipeline {
            gpu: self.gpu,
            pipeline: self.pipeline,
            bind_group,
        }
    }
}

pub struct ComputePipeline {
    gpu: Gpu,

    pipeline: wgpu::ComputePipeline,
    bind_group: BindGroup,
}

impl ComputePipeline {
    /// Dispatches the pipeline on the specified number of workgroups
    pub fn dispatch(&self, workgroups: Vector3<u32>) {
        self.gpu.dispatch(|encoder| {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, Some(&self.bind_group), &[]);
            compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
        });
    }
}

impl Gpu {
    /// Creates a new compute pipeline builder with the specified shader module.
    /// The compute entrypoint must be a function named `main`.
    pub fn compute_pipeline(&self, source: ShaderModuleDescriptor) -> ComputePipelineBuilder {
        let module = self.device.create_shader_module(source);

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
            gpu: self.clone(),
            pipeline,
            entries: Vec::new(),
        }
    }
}
