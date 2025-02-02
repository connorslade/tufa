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
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::RenderPipeline,
};

#[derive(ShaderType, Default)]
struct Uniform {
    window: Vector2<u32>,
    iters: u32,

    center: Vector2<f32>,
    zoom: f32,
    power: u32,
}

struct App {
    uniform: UniformBuffer<Uniform>,
    render: RenderPipeline,

    ctx: Uniform,
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        self.ctx.window = Vector2::new(window.width, window.height);

        self.uniform.upload(&self.ctx).unwrap();
        self.render.draw_quad(render_pass, 0..1);
    }

    fn ui(&mut self, gcx: GraphicsCtx, ctx: &Context) {
        let dragging_viewport = ctx.dragged_id().is_none();
        ctx.input(|input| {
            self.ctx.zoom += input.smooth_scroll_delta.y / 100.0;

            if input.pointer.any_down() && dragging_viewport {
                let window = gcx.window.inner_size();
                let zoom = 4.0 / self.ctx.zoom.exp();
                let delta = input.pointer.delta() * zoom;
                self.ctx.center +=
                    Vector2::new(-delta.x, delta.y) / window.width.min(window.height) as f32;
            }
        });

        egui::Window::new("Mandelbrot")
            .default_width(0.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(Slider::new(&mut self.ctx.zoom, 0.0..=12.0));
                    ui.label("Zoom");
                });

                ui.horizontal(|ui| {
                    ui.add(Slider::new(&mut self.ctx.iters, 0..=1000).step_by(1.0));
                    ui.label("Iters");
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.add(Slider::new(&mut self.ctx.power, 2..=8).step_by(2.0));
                    ui.label("Power");
                });

                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut self.ctx.center.x).speed(0.01));
                    ui.label("+");
                    ui.add(DragValue::new(&mut self.ctx.center.y).speed(0.01));
                    ui.label("i");
                    ui.label("Const");
                });
            });
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let uniform = gpu.create_uniform(&Uniform::default()).unwrap();
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

            ctx: Uniform {
                iters: 1000,
                power: 2,
                ..Uniform::default()
            },
        },
    )
    .run()?;

    Ok(())
}
