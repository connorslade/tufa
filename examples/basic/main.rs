use std::time::Instant;

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
    let gpu = Gpu::new()?;

    let buffer = gpu.create_storage(&Data { a: 10.0, b: 20.0 })?;

    let mut pipeline = gpu
        .compute_pipeline(include_wgsl!("compute.wgsl"))
        .bind_buffer(&buffer)
        .finish();

    let now = Instant::now();
    pipeline.dispatch_callback(Vector3::new(1, 1, 1), move || {
        let result = buffer.download().unwrap();
        println!("Result: {}. ({:?})", result.a, now.elapsed());
    });
    println!("Dispatched! ({:?})", now.elapsed());

    Ok(())
}
