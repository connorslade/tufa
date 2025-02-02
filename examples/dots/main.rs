use std::{ops::RangeInclusive, time::Instant};

use anyhow::{Ok, Result};
use compute::{
    buffer::{StorageBuffer, UniformBuffer},
    export::{
        egui::{emath::Numeric, Context, Slider, Ui, Window},
        nalgebra::{Vector2, Vector3},
        wgpu::{include_wgsl, RenderPass, ShaderStages},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    misc::mutability::Mutable,
    pipeline::{compute::ComputePipeline, render::RenderPipeline},
};
use encase::ShaderType;
use rand::{thread_rng, Rng};

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let ctx = Uniform {
        window: Vector2::zeros(),
        radius: 0.1,
        border: 1.0,
        speed: 1.0,
    };
    let uniform = gpu.create_uniform(&ctx)?;

    let dots = (0..100).map(|_| Particle::random()).collect::<Vec<_>>();
    let dots = gpu.create_storage(&dots)?;

    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .bind_buffer(&dots, ShaderStages::VERTEX_FRAGMENT)
        .bind_buffer(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .finish();
    let compute = gpu
        .compute_pipeline(include_wgsl!("compute.wgsl"))
        .bind_buffer(&dots)
        .bind_buffer(&uniform)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Dots Example"),
        App {
            render,
            compute,

            ctx,
            uniform,
            dots,

            last_frame: Instant::now(),
            dot_count: 100,
        },
    )
    .run()?;

    Ok(())
}

#[derive(ShaderType)]
struct Particle {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
}

#[derive(ShaderType)]
struct Uniform {
    window: Vector2<f32>,
    radius: f32,
    border: f32,
    speed: f32,
}

impl Particle {
    fn random() -> Self {
        let mut rand = thread_rng();
        Self {
            position: Vector2::new(rand.gen(), rand.gen()),
            velocity: (Vector2::new(rand.gen(), rand.gen()) * 2.0 - Vector2::repeat(1.0)) / 120.0,
        }
    }
}

struct App {
    render: RenderPipeline,
    compute: ComputePipeline,

    ctx: Uniform,
    uniform: UniformBuffer<Uniform>,
    dots: StorageBuffer<Vec<Particle>, Mutable>,

    last_frame: Instant,
    dot_count: u32,
}

impl Interactive for App {
    fn ui(&mut self, _gcx: GraphicsCtx, ctx: &Context) {
        Window::new("Dots")
            .default_width(0.0)
            .movable(false)
            .show(ctx, |ui| {
                ui.label(format!("Frame Time: {:.2?}", self.last_frame.elapsed()));
                self.last_frame = Instant::now();

                ui.separator();

                dragger(ui, "Radius", &mut self.ctx.radius, 0.0..=0.1);
                dragger(ui, "Border", &mut self.ctx.border, 1.0..=2.0);

                ui.separator();

                let mut dot_count = self.dot_count;
                dragger(ui, "Dots", &mut dot_count, 0..=65_535);
                dragger(ui, "Speed", &mut self.ctx.speed, 0.0..=1.0);

                if dot_count != self.dot_count {
                    self.dot_count = dot_count;
                    let dots = (0..dot_count)
                        .map(|_| Particle::random())
                        .collect::<Vec<_>>();
                    self.dots.upload(&dots).unwrap();
                }
            });
    }

    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let screen = gcx.window.inner_size();
        self.ctx.window = Vector2::new(screen.width as f32, screen.height as f32);

        self.uniform.upload(&self.ctx).unwrap();
        self.compute.dispatch(Vector3::new(self.dot_count, 1, 1));
        self.render.draw_quad(render_pass, 0..self.dot_count);
    }
}

fn dragger<T: Numeric>(ui: &mut Ui, label: &str, value: &mut T, range: RangeInclusive<T>) {
    ui.horizontal(|ui| {
        ui.add(Slider::new(value, range));
        ui.label(label);
    });
}
