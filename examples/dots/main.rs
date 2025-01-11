use anyhow::{Ok, Result};
use compute::{
    export::{
        nalgebra::{Vector2, Vector3},
        wgpu::{include_wgsl, RenderPass, ShaderStages},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::{compute::ComputePipeline, render::RenderPipeline},
};
use encase::ShaderType;
use rand::{thread_rng, Rng};

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let dots = (0..100).map(|_| Particle::random()).collect::<Vec<_>>();
    let dots = gpu.create_storage(dots)?;

    let compute = gpu
        .compute_pipeline(include_wgsl!("compute.wgsl"))
        .bind_buffer(&dots)
        .finish();
    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .bind_buffer(&dots, ShaderStages::VERTEX_FRAGMENT)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Dots Example"),
        App { render, compute },
    )
    .run()?;

    Ok(())
}

#[derive(ShaderType)]
struct Particle {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
}

impl Particle {
    fn random() -> Self {
        let mut rand = thread_rng();
        Self {
            position: Vector2::new(rand.gen(), rand.gen()),
            velocity: Vector2::new(rand.gen(), rand.gen()) / 120.0,
        }
    }
}

struct App {
    render: RenderPipeline,
    compute: ComputePipeline,
}

impl Interactive for App {
    fn render(&mut self, _gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        self.compute.dispatch(Vector3::new(100, 0, 0));
        self.render.draw_quad(render_pass, 0..100);
    }
}
