use anyhow::Result;

use compute::{
    buffer::UniformBuffer,
    export::{
        egui::{self, Context, Slider},
        encase::ShaderType,
        nalgebra::{Vector2, Vector3},
        wgpu::{include_wgsl, RenderPass, ShaderStages},
        winit::{dpi::LogicalSize, window::WindowAttributes},
    },
    gpu::Gpu,
    interactive::Interactive,
    pipeline::{compute::ComputePipeline, render::RenderPipeline},
};

const SIZE: Vector2<u32> = Vector2::new(4096, 4096);

#[derive(ShaderType, Default)]
struct Uniform {
    size: Vector2<u32>,
    zoom: f32,
}

struct App {
    compute: ComputePipeline,
    uniform: UniformBuffer<Uniform>,
    render: RenderPipeline,

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
        self.render.draw_screen_quad(render_pass);
    }

    fn ui(&mut self, ctx: &Context) {
        ctx.input(|input| self.zoom += input.smooth_scroll_delta.y / 100.0);

        egui::Window::new("Mandelbrot").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Zoom");
                ui.add(Slider::new(&mut self.zoom, 0.0..=12.0))
            });
        });
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let uniform = gpu.create_uniform(Uniform::default()).unwrap();
    let buffer = gpu
        .create_storage(vec![0; (SIZE.x * SIZE.y) as usize])
        .unwrap();

    let compute = gpu
        .compute_pipeline(include_wgsl!("compute.wgsl"))
        .bind_buffer(&uniform)
        .bind_buffer(&buffer)
        .finish();

    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .bind_buffer(&uniform, ShaderStages::FRAGMENT)
        .bind_buffer(&buffer, ShaderStages::FRAGMENT)
        .finish();

    gpu.create_window(
        WindowAttributes::default()
            .with_title("Mandelbrot")
            .with_inner_size(LogicalSize::new(1920, 1080)),
        App {
            compute,
            uniform,
            render,
            zoom: 0.0,
        },
    )
    .run()?;

    Ok(())
}
