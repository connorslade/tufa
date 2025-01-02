use anyhow::Result;

use compute::{
    buffer::{IndexBuffer, StorageBuffer, UniformBuffer, VertexBuffer},
    export::{
        egui::{self, Context, Slider},
        encase::ShaderType,
        nalgebra::{Vector2, Vector3},
        wgpu::{include_wgsl, RenderPass, ShaderStages},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::Interactive,
    pipeline::{
        compute::ComputePipeline,
        render::{RenderPipeline, Vertex, QUAD_INDEX, QUAD_VERTEX},
    },
};

#[derive(ShaderType)]
struct Uniform {
    size: Vector2<u32>,
    zoom: f32,
}

const SIZE: Vector2<u32> = Vector2::new(4096, 4096);

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let uniform = gpu.create_uniform(Uniform {
        size: SIZE,
        zoom: 0.0,
    })?;
    let buffer = gpu.create_storage(vec![0; (SIZE.x * SIZE.y) as usize])?;

    let compute = gpu
        .compute_pipeline(include_wgsl!("shader.wgsl"))
        .bind_buffer(&uniform)
        .bind_buffer(&buffer)
        .finish();
    compute.dispatch(Vector3::new(SIZE.x / 8, SIZE.y / 8, 1));

    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .bind_buffer(&uniform, ShaderStages::FRAGMENT)
        .bind_buffer(&buffer, ShaderStages::FRAGMENT)
        .finish();
    let vertex = gpu.create_vertex(QUAD_VERTEX)?;
    let index = gpu.create_index(QUAD_INDEX)?;

    gpu.create_window(
        WindowAttributes::default().with_title("Mandelbrot"),
        App {
            compute,
            uniform,
            buffer,

            render,
            vertex,
            index,

            zoom: 0.0,
        },
    )
    .run()?;

    Ok(())
}

struct App {
    compute: ComputePipeline,
    uniform: UniformBuffer<Uniform>,
    buffer: StorageBuffer<Vec<u32>>,

    render: RenderPipeline,
    vertex: VertexBuffer<Vertex>,
    index: IndexBuffer,

    zoom: f32,
}

impl Interactive for App {
    fn render(&mut self, _gpu: &Gpu, render_pass: &mut RenderPass) {
        self.uniform
            .upload(Uniform {
                size: SIZE,
                zoom: self.zoom,
            })
            .unwrap();
        self.compute
            .dispatch(Vector3::new(SIZE.x / 8, SIZE.y / 8, 1));

        self.render
            .dispatch(render_pass, &self.index, &self.vertex, 0..6);
    }

    fn ui(&mut self, ctx: &Context) {
        egui::Window::new("Mandelbrot").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Zoom");
                ui.add(Slider::new(&mut self.zoom, 0.0..=12.0))
            });
        });
    }
}
