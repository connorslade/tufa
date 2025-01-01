# `compute`

A wgpu abstraction layer for simplified integration of compute shaders.

## Example

```rust
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
```

```wgsl
@group(0)
@binding(0)
var<storage, read_write> data: Data;

struct Data {
    a: f32,
    b: f32,
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    data.a = data.a + data.b;
}
```
