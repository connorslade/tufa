use anyhow::Result;

use compute::{
    export::{encase::ShaderType, nalgebra::Vector3, wgpu::ShaderSource},
    gpu::Gpu,
};
use image::Rgb;

#[derive(ShaderType)]
struct Data {
    data: [[f32; 3]; 128 * 128],
}

fn main() -> Result<()> {
    let mut gpu = Gpu::init()?;

    let buffer = gpu.create_buffer_init(Data::default())?;

    let pipeline = gpu
        .compute_pipeline(ShaderSource::Wgsl(include_str!("shader.wgsl").into()))
        .bind_buffer(&buffer)
        .finish();

    pipeline.dispatch(&mut gpu, Vector3::new(128, 128, 1));

    let result = buffer.download(&mut gpu)?;

    let mut img = image::ImageBuffer::new(128, 128);
    for x in 0..128 {
        for y in 0..128 {
            let color = result.data[y * 128 + x];
            img.put_pixel(
                x as u32,
                y as u32,
                Rgb([
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                ]),
            );
        }
    }

    img.save("out.png")?;

    Ok(())
}

impl Default for Data {
    fn default() -> Self {
        Self {
            data: [[0.0; 3]; 128 * 128],
        }
    }
}
