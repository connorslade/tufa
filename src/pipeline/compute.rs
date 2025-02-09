use nalgebra::Vector3;
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ComputePassDescriptor,
    ComputePipelineDescriptor, PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderStages,
};

use crate::{
    bindings::{Bindable, BindableResourceId},
    gpu::Gpu,
    misc::ids::PipelineId,
};

use super::PipelineStatus;

pub struct ComputePipeline {
    gpu: Gpu,

    id: PipelineId,
    pipeline: wgpu::ComputePipeline,
    entries: Vec<BindableResourceId>,
    bind_group: BindGroup,
}

pub struct ComputePipelineBuilder {
    gpu: Gpu,

    module: ShaderModule,
    bind_group_layout: Vec<BindGroupLayoutEntry>,
    entries: Vec<BindableResourceId>,
}

impl ComputePipeline {
    /// Dispatches the pipeline on the specified number of workgroups
    pub fn dispatch(&mut self, workgroups: Vector3<u32>) {
        self.dispatch_inner(workgroups, true);
    }

    pub fn dispatch_callback(
        &mut self,
        workgroups: Vector3<u32>,
        callback: impl FnOnce() + Send + 'static,
    ) {
        self.dispatch_callback_inner(workgroups, callback, true);
    }

    /// Queues the compute shader to run with the next compute dispach, render pass, or call to [`Gpu::flush_dispatch_queue`].
    pub fn queue_dispatch(&mut self, workgroups: Vector3<u32>) {
        self.dispatch_inner(workgroups, false);
    }

    pub fn queue_dispatch_callback(
        &mut self,
        workgroups: Vector3<u32>,
        callback: impl FnOnce() + Send + 'static,
    ) {
        self.dispatch_callback_inner(workgroups, callback, false);
    }

    fn recreate_bind_group(&mut self) {
        if self.gpu.binding_manager.get_pipeline(self.id).dirty {
            self.bind_group = self.gpu.binding_manager.create_bind_group(
                &self.gpu.device,
                &self.pipeline.get_bind_group_layout(0),
                &self.entries,
            )
        }
    }

    fn dispatch_inner(&mut self, workgroups: Vector3<u32>, immediate: bool) {
        self.recreate_bind_group();
        self.gpu.dispach(
            |encoder| {
                let mut compute_pass =
                    encoder.begin_compute_pass(&ComputePassDescriptor::default());
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, Some(&self.bind_group), &[]);
                compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
            },
            immediate,
        );
    }

    fn dispatch_callback_inner(
        &mut self,
        workgroups: Vector3<u32>,
        callback: impl FnOnce() + Send + 'static,
        immediate: bool,
    ) {
        self.recreate_bind_group();
        self.gpu.dispach_callback(
            |encoder| {
                let mut compute_pass =
                    encoder.begin_compute_pass(&ComputePassDescriptor::default());
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, Some(&self.bind_group), &[]);
                compute_pass.dispatch_workgroups(workgroups.x, workgroups.y, workgroups.z);
            },
            callback,
            immediate,
        );
    }
}

impl ComputePipelineBuilder {
    /// Adds the supplied buffer as the next entry in the bind group, starting with binding zero and counting up.
    pub fn bind(mut self, entry: &impl Bindable) -> Self {
        self.entries.push(entry.resource_id());
        self.bind_group_layout.push(BindGroupLayoutEntry {
            binding: self.bind_group_layout.len() as u32,
            visibility: ShaderStages::COMPUTE,
            ty: entry.binding_type(),
            count: entry.count(),
        });

        self
    }

    /// Converts the pipeline builder into an actual compte pipeline
    pub fn finish(self) -> ComputePipeline {
        let device = &self.gpu.device;
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &self.bind_group_layout,
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &self.module,
            entry_point: Some("main"),
            // todo: pass in constants?
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let id = PipelineId::new();
        self.gpu.binding_manager.add_pipeline(
            id,
            PipelineStatus {
                resources: self.entries.clone(),
                dirty: false,
            },
        );

        ComputePipeline {
            id,
            bind_group: self.gpu.binding_manager.create_bind_group(
                &self.gpu.device,
                &pipeline.get_bind_group_layout(0),
                &self.entries,
            ),
            gpu: self.gpu,
            entries: self.entries,
            pipeline,
        }
    }
}

impl Gpu {
    /// Creates a new compute pipeline builder with the specified shader module.
    /// The compute entrypoint must be a function named `main`.
    pub fn compute_pipeline(&self, source: ShaderModuleDescriptor) -> ComputePipelineBuilder {
        let module = self.device.create_shader_module(source);

        ComputePipelineBuilder {
            gpu: self.clone(),

            module,
            bind_group_layout: Vec::new(),
            entries: Vec::new(),
        }
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_pipeline(self.id);
    }
}
