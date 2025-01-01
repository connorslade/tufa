use anyhow::Result;

use compute::{
    export::{encase::ShaderType, nalgebra::Vector3, wgpu::ShaderSource},
    Gpu,
};

#[derive(ShaderType)]
struct Data {
    a: f32,
    b: f32,
}

fn main() -> Result<()> {
    let mut gpu = Gpu::init()?;

    let buffer = gpu.create_buffer_init(Data { a: 10.0, b: 20.0 })?;

    let pipeline = gpu
        .compute_pipeline(ShaderSource::Wgsl(include_str!("shader.wgsl").into()))
        .bind_buffer(&buffer)
        .finish();

    pipeline.dispatch(&mut gpu, Vector3::new(1, 1, 1));

    let result = buffer.download(&mut gpu)?;
    println!("Result: {}", result.a);

    Ok(())
}
