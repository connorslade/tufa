use anyhow::{Ok, Result};
use encase::ShaderType;
use nalgebra::{Vector2, Vector3};
use tufa::{
    bindings::buffer::{IndexBuffer, VertexBuffer},
    export::{
        nalgebra::Vector4,
        wgpu::{include_wgsl, RenderPass},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::RenderPipeline,
};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

const VERTEX_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: 32,
    step_mode: VertexStepMode::Vertex,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        },
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 4 * 4,
            shader_location: 1,
        },
    ],
};

struct App {
    pipeline: RenderPipeline,
    vertex: VertexBuffer<Vertex>,
    index: IndexBuffer,
}

#[derive(ShaderType)]
struct Vertex {
    position: Vector4<f32>,
    bary: Vector3<f32>,
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let pipeline = gpu
        .render_pipeline(include_wgsl!("shader.wgsl"))
        .vertex_layout(VERTEX_LAYOUT)
        .finish();

    let vertex = gpu.create_vertex(&[
        Vertex::new(Vector4::new(-0.5, -0.75, 0.0, 1.0), Vector3::x()),
        Vertex::new(Vector4::new(-0.5, 0.2, 0.0, 1.0), Vector3::z()),
        Vertex::new(Vector4::new(0.5, 0.75, 0.0, 1.0), Vector3::y()),
    ]);
    let index = gpu.create_index(&[0, 1, 2]);

    gpu.create_window(
        WindowAttributes::default(),
        App {
            pipeline,
            vertex,
            index,
        },
    )
    .run()?;

    Ok(())
}

impl Interactive for App {
    fn render(&mut self, _gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        self.pipeline
            .draw(render_pass, &self.index, &self.vertex, 0..3);
    }

    fn ui(&mut self, gcx: GraphicsCtx, ctx: &egui::Context) {
        let window = gcx.window.inner_size();
        let cursor = ctx.input(|i| i.pointer.latest_pos().unwrap_or_default());

        let normalized = Vector2::new(
            cursor.x / window.width as f32,
            cursor.y / window.height as f32,
        );

        self.vertex.upload(&[
            Vertex::new(Vector4::new(-0.5, -0.75, 0.0, 1.0), Vector3::x()),
            Vertex::new(
                Vector4::new(normalized.x, normalized.y, 0.0, 1.0),
                Vector3::z(),
            ),
            Vertex::new(Vector4::new(0.5, 0.75, 0.0, 1.0), Vector3::y()),
        ]);
    }
}

impl Vertex {
    pub fn new(position: Vector4<f32>, bary: Vector3<f32>) -> Self {
        Self { position, bary }
    }
}
