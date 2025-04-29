use std::{f32::consts::TAU, time::Instant};

use anyhow::{Ok, Result};
use encase::ShaderType;
use image::{ImageBuffer, Rgb};
use tufa::{
    bindings::mutability::Immutable,
    export::{
        nalgebra::{Matrix4, Vector2, Vector3, Vector4},
        wgpu::{include_wgsl, ShaderStages},
    },
    gpu::Gpu,
    misc::camera::PerspectiveCamera,
    pipeline::render::Vertex,
};

const FRAMES: u32 = 120;
const SIZE: Vector2<u32> = Vector2::new(1024, 1024);

#[derive(ShaderType, Default)]
struct Uniform {
    transformation: Matrix4<f32>,
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let camera = PerspectiveCamera::default().with_position(-Vector3::z());
    let uniform = gpu.create_uniform(&Uniform::default())?;
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

    let texture = gpu.create_texture_2d(SIZE);
    let depth = gpu.create_texture_2d(SIZE);
    let staging = gpu.create_storage_empty::<_, Immutable>(4 * (SIZE.x * SIZE.y) as u64);

    let start = Instant::now();
    for i in 0..FRAMES {
        let t = i as f32 / FRAMES as f32;
        let rotation = Matrix4::new_rotation(Vector3::new(0.0, t * TAU, 0.0));
        let transformation = camera.view_projection(1.0) * rotation;

        uniform.upload(&Uniform { transformation });
        gpu.render_pass(&texture, &depth, |render_pass| {
            render.draw(render_pass, &index, &vertex, 0..3)
        });

        texture.copy_to_buffer(&staging);

        staging.download_async(move |result| {
            ImageBuffer::from_par_fn(SIZE.x, SIZE.y, |x, y| {
                let color = result[(y * SIZE.x + x) as usize];
                Rgb([color as u8, (color >> 8) as u8, (color >> 16) as u8])
            })
            .save(format!("out/{i:0>3}-out.png"))
            .unwrap();
        });
    }

    let elapsed = start.elapsed();
    println!("FPS: {:.2}", FRAMES as f32 / elapsed.as_secs_f32() as f32);

    Ok(())
}
