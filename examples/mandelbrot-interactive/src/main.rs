use anyhow::Result;

use compute::{
    buffer::UniformBuffer,
    export::{
        egui::{self, Context, DragValue, Slider},
        encase::ShaderType,
        nalgebra::Vector2,
        wgpu::{include_wgsl, RenderPass, ShaderStages},
        winit::{dpi::LogicalSize, window::WindowAttributes},
    },
    gpu::Gpu,
    interactive::Interactive,
    pipeline::render::RenderPipeline,
};

#[derive(ShaderType, Default)]
struct Uniform {
    size: Vector2<u32>,
    center: Vector2<f32>,
    zoom: f32,
}

struct App {
    uniform: UniformBuffer<Uniform>,
    render: RenderPipeline,

    ctx: Uniform,
}

impl Interactive for App {
    fn render(&mut self, _gpu: &Gpu, render_pass: &mut RenderPass) {
        self.uniform.upload(&self.ctx).unwrap();
        self.render.draw_screen_quad(render_pass);
    }

    fn ui(&mut self, ctx: &Context) {
        ctx.input(|input| {
            self.ctx.zoom += input.smooth_scroll_delta.y / 100.0;

            if input.pointer.any_down() && input.modifiers.shift {
                let zoom = 4.0 / self.ctx.zoom.exp();
                let delta = input.pointer.delta() * zoom / 100.0;
                self.ctx.center += Vector2::new(-delta.x, delta.y);
            }
        });

        egui::Window::new("Mandelbrot").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Zoom");
                ui.add(Slider::new(&mut self.ctx.zoom, 0.0..=12.0))
            });

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut self.ctx.center.x).speed(0.01));
                ui.label("+");
                ui.add(DragValue::new(&mut self.ctx.center.y).speed(0.01));
                ui.label("i");
            })
        });
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let uniform = gpu.create_uniform(Uniform::default()).unwrap();
    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .bind_buffer(&uniform, ShaderStages::FRAGMENT)
        .finish();

    gpu.create_window(
        WindowAttributes::default()
            .with_title("Mandelbrot")
            .with_inner_size(LogicalSize::new(1920, 1080)),
        App {
            uniform,
            render,

            ctx: Uniform::default(),
        },
    )
    .run()?;

    Ok(())
}
