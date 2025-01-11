use anyhow::Result;

use compute::{
    export::{encase::ShaderType, nalgebra::Vector3, wgpu::include_wgsl},
    gpu::Gpu,
};

#[derive(ShaderType)]
struct Data {
    a: f32,
    b: f32,
}

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let buffer = gpu.create_storage(Data { a: 10.0, b: 20.0 })?;

    let mut pipeline = gpu
        .compute_pipeline(include_wgsl!("compute.wgsl"))
        .bind_buffer(&buffer)
        .finish();

    pipeline.dispatch(Vector3::new(1, 1, 1));

    let result = buffer.download()?;
    println!("Result: {}", result.a);

    Ok(())
}
