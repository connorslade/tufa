use std::ops::Range;

use consts::VERTEX_BUFFER_LAYOUT;
use encase::ShaderType;
use nalgebra::{Vector2, Vector4};
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BlendComponent, BlendState,
    ColorTargetState, ColorWrites, FragmentState, IndexFormat, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, RenderPass, ShaderModule,
    ShaderModuleDescriptor, ShaderStages, VertexBufferLayout, VertexState,
};

use crate::{
    bindings::{Bindable, BindableResourceId, IndexBuffer, VertexBuffer},
    gpu::Gpu,
    misc::ids::PipelineId,
    TEXTURE_FORMAT,
};

use super::PipelineStatus;
pub mod consts;

#[derive(ShaderType)]
pub struct Vertex {
    pub position: Vector4<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub const fn new(position: Vector4<f32>, uv: Vector2<f32>) -> Self {
        Self { position, uv }
    }
}

pub struct RenderPipeline {
    gpu: Gpu,

    id: PipelineId,
    pipeline: wgpu::RenderPipeline,
    entries: Vec<BindableResourceId>,
    bind_group: BindGroup,
}

pub struct RenderPipelineBuilder<'a> {
    gpu: Gpu,

    module: ShaderModule,
    vertex_layout: Option<VertexBufferLayout<'a>>,
    bind_group_layout: Vec<BindGroupLayoutEntry>,
    bind_group: Vec<BindableResourceId>,
}

impl RenderPipeline {
    fn recreate_bind_group(&mut self) {
        if self.gpu.binding_manager.get_pipeline(self.id).dirty {
            self.bind_group = self.gpu.binding_manager.create_bind_group(
                &self.gpu.device,
                &self.pipeline.get_bind_group_layout(0),
                &self.entries,
            );
        }
    }

    pub fn draw(
        &mut self,
        render_pass: &mut RenderPass,
        index: &IndexBuffer,
        vertex: &VertexBuffer<Vertex>,
        indices: Range<u32>,
    ) {
        self.recreate_bind_group();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, Some(&self.bind_group), &[]);
        render_pass.set_index_buffer(index.get().slice(..), IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, vertex.get().slice(..));
        render_pass.draw_indexed(indices, 0, 0..1);
    }

    pub fn draw_quad(&mut self, render_pass: &mut RenderPass, instances: Range<u32>) {
        self.recreate_bind_group();
        let (vertex, index) = self.gpu.default_buffers();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, Some(&self.bind_group), &[]);
        render_pass.set_index_buffer(index.get().slice(..), IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, vertex.get().slice(..));
        render_pass.draw_indexed(0..6, 0, instances);
    }

    pub fn instance_quad<T>(
        &mut self,
        render_pass: &mut RenderPass,
        instances: &VertexBuffer<T>,
        range: Range<u32>,
    ) {
        let (vertex, index) = self.gpu.default_buffers();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, Some(&self.bind_group), &[]);
        render_pass.set_index_buffer(index.get().slice(..), IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, vertex.get().slice(..));
        render_pass.set_vertex_buffer(1, instances.get().slice(..));
        render_pass.draw_indexed(0..6, 0, range);
    }
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn bind(mut self, entry: &impl Bindable, visibility: ShaderStages) -> Self {
        let binding = self.bind_group.len() as u32;

        self.bind_group.push(entry.resource_id());
        self.bind_group_layout.push(BindGroupLayoutEntry {
            binding,
            visibility,
            ty: entry.binding_type(),
            count: entry.count(),
        });

        self
    }

    pub fn instance_layout(mut self, layout: VertexBufferLayout<'a>) -> Self {
        self.vertex_layout = Some(layout);
        self
    }

    pub fn finish(self) -> RenderPipeline {
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

        let mut vertex_buffers = vec![VERTEX_BUFFER_LAYOUT];
        if let Some(layout) = self.vertex_layout {
            vertex_buffers.push(layout);
        }

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &self.module,
                entry_point: Some("vert"),
                buffers: &vertex_buffers,
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &self.module,
                entry_point: Some("frag"),
                targets: &[Some(ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState {
                        color: BlendComponent::OVER,
                        alpha: BlendComponent::OVER,
                    }),
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let bind_group = self.gpu.binding_manager.create_bind_group(
            &self.gpu.device,
            &pipeline.get_bind_group_layout(0),
            &self.bind_group,
        );

        let id = PipelineId::new();
        self.gpu.binding_manager.add_pipeline(
            id,
            PipelineStatus {
                resources: self.bind_group.clone(),
                dirty: false,
            },
        );

        RenderPipeline {
            gpu: self.gpu,
            id,
            pipeline,
            bind_group,
            entries: self.bind_group,
        }
    }
}

impl Gpu {
    pub fn render_pipeline(&self, source: ShaderModuleDescriptor) -> RenderPipelineBuilder {
        let module = self.device.create_shader_module(source);

        RenderPipelineBuilder {
            gpu: self.clone(),
            module,
            vertex_layout: None,
            bind_group_layout: Vec::new(),
            bind_group: Vec::new(),
        }
    }
}

impl Drop for RenderPipeline {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_pipeline(self.id);
    }
}
