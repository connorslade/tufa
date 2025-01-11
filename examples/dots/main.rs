use anyhow::{Ok, Result};
use compute::{
    buffer::VertexBuffer,
    export::{
        nalgebra::Vector2,
        wgpu::{
            include_wgsl, RenderPass, VertexAttribute, VertexBufferLayout, VertexFormat,
            VertexStepMode,
        },
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::RenderPipeline,
};
use encase::ShaderType;
use rand::{thread_rng, Rng};

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let dots = (0..100).map(|_| Dot::random()).collect::<Vec<_>>();
    let dots = gpu.create_vertex(&dots)?;

    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .instance_layout(VertexBufferLayout {
            array_stride: 16,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 2,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    offset: 8,
                    shader_location: 3,
                },
            ],
        })
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Dots Example"),
        App { render, dots },
    )
    .run()?;

    Ok(())
}

#[derive(ShaderType)]
struct Dot {
    position: Vector2<f32>,
    radius: f32,
}

impl Dot {
    fn random() -> Self {
        let mut rand = thread_rng();
        Self {
            position: Vector2::new(rand.gen::<f32>() * 2.0 - 1.0, rand.gen::<f32>() * 2.0 - 1.0),
            radius: rand.gen::<f32>() / 10.0,
        }
    }
}

struct App {
    render: RenderPipeline,
    dots: VertexBuffer<Dot>,
}

impl Interactive for App {
    fn render(&mut self, _gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        self.render.instance_quad(render_pass, &self.dots, 0..100);
    }
}
