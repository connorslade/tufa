use nalgebra::Vector3;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, ComputePassDescriptor,
    ComputePipelineDescriptor, PipelineCompilationOptions, ShaderModuleDescriptor,
};

use crate::{
    buffer::{Bindable, BindableResource},
    gpu::Gpu,
    misc::ids::PipelineId,
};

use super::PipelineStatus;

pub struct ComputePipeline {
    gpu: Gpu,

    id: PipelineId,
    pipeline: wgpu::ComputePipeline,
    entries: Vec<BindableResource>,
    bind_group: BindGroup,
}

pub struct ComputePipelineBuilder {
    gpu: Gpu,

    pipeline: wgpu::ComputePipeline,
    entries: Vec<BindableResource>,
}

impl ComputePipeline {
    /// Dispatches the pipeline on the specified number of workgroups
    pub fn dispatch(&mut self, workgroups: Vector3<u32>) {
        if self.gpu.pipelines.read()[&self.id].dirty {
            self.bind_group = create_bind_group(&self.gpu, &self.pipeline, &self.entries);
        }

        self.gpu.dispatch(|encoder| {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, Some(&self.bind_group), &[]);
            compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
        });
    }
}

impl ComputePipelineBuilder {
    /// Adds the supplied buffer as the next entry in the bind group, starting with binding zero and counting up.
    pub fn bind_buffer(mut self, entry: &impl Bindable) -> Self {
        self.entries.push(entry.resource());
        self
    }

    /// Converts the pipeline builder into an actual compte pipeline
    pub fn finish(self) -> ComputePipeline {
        let id = PipelineId::new();
        self.gpu.pipelines.write().insert(
            id,
            PipelineStatus {
                resources: self.entries.clone(),
                dirty: false,
            },
        );

        ComputePipeline {
            id,
            bind_group: create_bind_group(&self.gpu, &self.pipeline, &self.entries),
            gpu: self.gpu,
            pipeline: self.pipeline,
            entries: self.entries,
        }
    }
}

fn create_bind_group(
    gpu: &Gpu,
    pipeline: &wgpu::ComputePipeline,
    entries: &[BindableResource],
) -> BindGroup {
    let buffers = gpu.buffers.read();
    gpu.device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries
            .iter()
            .enumerate()
            .map(|(binding, id)| BindGroupEntry {
                binding: binding as u32,
                resource: match id {
                    BindableResource::Buffer(buffer) => buffers[buffer].as_entire_binding(),
                },
            })
            .collect::<Vec<_>>(),
    })
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

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        self.gpu.pipelines.write().remove(&self.id);
    }
}
