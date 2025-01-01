use anyhow::Result;

use compute::{
    export::{
        encase::ShaderType,
        nalgebra::{Vector2, Vector3},
        wgpu::ShaderSource,
    },
    gpu::Gpu,
};
use image::{Pixel, Rgb};

#[derive(ShaderType)]
struct Data {
    size: Vector2<u32>,
    zoom: f32,
    #[size(runtime)]
    data: Vec<Vector3<f32>>,
}

const SIZE: Vector2<u32> = Vector2::new(4096, 4096);
const ZOOM: f32 = 0.0;

fn main() -> Result<()> {
    let mut gpu = Gpu::init()?;

    let buffer = gpu.create_buffer(Data {
        size: SIZE,
        zoom: ZOOM,
        data: vec![Vector3::zeros(); (SIZE.x * SIZE.y) as usize],
    })?;

    let pipeline = gpu
        .compute_pipeline(ShaderSource::Wgsl(include_str!("shader.wgsl").into()))
        .bind_buffer(&buffer)
        .finish();

    for zoom_idx in 0..10_00 {
        let zoom = zoom_idx as f32 / 100.0;
        buffer.upload(
            &mut gpu,
            Data {
                size: SIZE,
                zoom: zoom,
                data: vec![Vector3::zeros(); (SIZE.x * SIZE.y) as usize],
            },
        )?;

        pipeline.dispatch(&mut gpu, Vector3::new(SIZE.x / 8, SIZE.y / 8, 1));

        let result = buffer.download(&mut gpu)?;

        let mut img = image::ImageBuffer::new(SIZE.x, SIZE.y);
        for x in 0..SIZE.x as usize {
            for y in 0..SIZE.y as usize {
                let color = result.data[y * SIZE.x as usize + x];
                let pixel = *Rgb::from_slice(color.map(|x| (x * 255.0) as u8).as_slice());
                img.put_pixel(x as u32, y as u32, pixel);
            }
        }

        img.save(format!("rec/out-{zoom_idx:0>4}.png"))?;
    }

    Ok(())
}
