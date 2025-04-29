use anyhow::{Ok, Result};
use encase::ShaderType;
use image::{ImageBuffer, Rgb};
use tufa::{
    bindings::{IndexBuffer, UniformBuffer, VertexBuffer},
    export::{
        nalgebra::{Matrix4, Vector2, Vector3, Vector4},
        wgpu::{include_wgsl, ShaderStages},
    },
    gpu::Gpu,
    misc::camera::PerspectiveCamera,
    pipeline::render::{RenderPipeline, Vertex},
};

#[derive(ShaderType, Default)]
struct Uniform {
    transformation: Matrix4<f32>,
}

struct App {
    camera: PerspectiveCamera,
    uniform: UniformBuffer<Uniform>,
    vertex: VertexBuffer<Vertex>,
    index: IndexBuffer,
    render: RenderPipeline,
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let camera = PerspectiveCamera::default().with_position(-Vector3::z());
    let uniform = gpu.create_uniform(&Uniform {
        transformation: camera.view_projection(1.0),
    })?;
    let vertex = gpu.create_vertex(&[
        Vertex::new(Vector4::new(0.0, 0.5, 0.0, 1.0), Vector2::new(0.0, 1.0)),
        Vertex::new(Vector4::new(-0.5, -0.5, 0.0, 1.0), Vector2::new(0.0, 0.0)),
        Vertex::new(Vector4::new(0.5, -0.5, 0.0, 1.0), Vector2::new(1.0, 0.0)),
    ])?;
    let index = gpu.create_index(&[0, 1, 2]);
    let mut render = gpu
        .render_pipeline(include_wgsl!("shader.wgsl"))
        .bind(&uniform, ShaderStages::VERTEX)
        .finish();

    let texture = gpu.create_texture_2d(Vector2::repeat(1024));
    let depth = gpu.create_texture_2d(Vector2::repeat(1024));

    gpu.render_pass(&texture, &depth, |render_pass| {
        render.draw(render_pass, &index, &vertex, 0..3);
    });

    let out = gpu.create_storage(&vec![0; 1024 * 1024]).unwrap();
    texture.copy_to_buffer(&out);

    let result = out.download().unwrap();
    ImageBuffer::from_par_fn(1024, 1024, |x, y| {
        let color = result[(y * 1024 + x) as usize];
        Rgb([color as u8, (color >> 8) as u8, (color >> 16) as u8])
    })
    .save("out.png")
    .unwrap();

    Ok(())
}
