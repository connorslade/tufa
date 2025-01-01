use anyhow::Result;

use compute::{
    export::{
        encase::ShaderType,
        nalgebra::{Vector2, Vector3},
        wgpu::ShaderSource,
    },
    gpu::Gpu,
};
use image::{ImageBuffer, Rgb};

#[derive(ShaderType)]
struct Uniform {
    size: Vector2<u32>,
    zoom: f32,
}

const SIZE: Vector2<u32> = Vector2::new(4096, 4096);

fn main() -> Result<()> {
    let mut gpu = Gpu::init()?;

    let uniform = gpu.create_uniform(Uniform {
        size: SIZE,
        zoom: 0.0,
    })?;
    let buffer = gpu.create_storage(vec![0; (SIZE.x * SIZE.y) as usize])?;

    let pipeline = gpu
        .compute_pipeline(ShaderSource::Wgsl(include_str!("shader.wgsl").into()))
        .bind_buffer(&uniform)
        .bind_buffer(&buffer)
        .finish();

    for zoom in 0..10_0 {
        uniform.upload(
            &gpu,
            Uniform {
                size: SIZE,
                zoom: zoom as f32 / 10.0,
            },
        )?;

        pipeline.dispatch(&gpu, Vector3::new(SIZE.x / 8, SIZE.y / 8, 1));

        let result = buffer.download(&gpu)?;

        let img = ImageBuffer::from_par_fn(SIZE.x, SIZE.y, |x, y| {
            let color = result[(y * SIZE.x + x) as usize];

            let r = (color & 0xFF) as u8;
            let g = (color >> 8 & 0xFF) as u8;
            let b = (color >> 16 & 0xFF) as u8;

            Rgb([r, g, b])
        });

        img.save(format!("rec/out-{zoom:0>4}.png"))?;
    }

    Ok(())
}
