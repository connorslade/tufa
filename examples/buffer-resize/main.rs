use anyhow::Result;

use compute::{
    export::{encase::ShaderType, nalgebra::Vector3, wgpu::include_wgsl},
    gpu::Gpu,
};

#[derive(ShaderType)]
struct Data {
    out: f32,
    #[size(runtime)]
    items: Vec<f32>,
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let buffer = gpu.create_storage(&Data {
        out: 0.0,
        items: vec![1.0, 2.0],
    })?;

    let mut pipeline = gpu
        .compute_pipeline(include_wgsl!("compute.wgsl"))
        .bind_buffer(&buffer)
        .finish();

    buffer.upload(&Data {
        out: 0.0,
        items: vec![1.0, 2.0, 3.0, 4.0],
    })?;

    pipeline.dispatch(Vector3::new(1, 1, 1));

    let result = buffer.download()?;
    println!("Result: {}", result.out);

    Ok(())
}

/*0.8779509120,
0.8912942246,
0.1590277469,
0.2930850595,
0.0145279013,
0.7886976515,
0.9350796225,
0.8704332947,
0.2262764496,
0.5275122512, */
