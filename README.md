# `tufa` [![Crates.io Version](https://img.shields.io/crates/v/tufa)](https://crates.io/crate/tufa) [![docs.rs](https://img.shields.io/docsrs/tufa)](https://docs.rs/tufa)

A wgpu abstraction layer.
Was originally just for compute shaders, but now has support for basic rendering that often comes up when making simulations, like egui UIs and quad rendering.
Also, it has support for ray-tracing acceleration structures.

See [connorslade/ray-tracing](https://github.com/connorslade/ray-tracing), [examples/mandelbrot-interactive](https://github.com/connorslade/compute/tree/main/examples/mandelbrot-interactive), and [connorslade/physics](https://github.com/connorslade/physics) for some neat examples.

## Example

<img src="https://github.com/user-attachments/assets/0a89db41-d732-4701-8955-4bbcb4181b19" width="25%" align="right" >

This example shows a program that generates zoom animations through the [Mandelbrot Set](https://en.wikipedia.org/wiki/Mandelbrot_set).
Each frame is rendered and colored on the GPU then asynchronously downloaded back to the CPU to be saved as a PNG image.
The GIF on the right shows a zoom from 0 to 15, at the end you can see the limited precision of 32 bit floats.

If this library were not used, this example would probably need about 300 lines of rust to init the GPU, pipelines, buffers, bind groups, etc.

<details>
<summary>WGSL Shader</summary>

```wgsl
@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> data: array<u32>;

struct Uniform {
    size: vec2<u32>,
    zoom: f32
}

const PI: f32 = 3.1415926538;

const N: i32 = 1000;
const POI: vec2<f32> = vec2(-1.7864323556423187, -2.905726428359401e-7);

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {
    var zoom = 4.0 / exp(ctx.zoom);
    var c = (vec2(f32(pos.x), f32(pos.y)) / f32(ctx.size.x) - 0.5) * zoom + POI;

    var color = evaluate(c);
    var packed = pack4x8unorm(vec4(color, 0.0));

    data[ctx.size.x * pos.y + pos.x] = packed;
}

fn evaluate(c: vec2<f32>) -> vec3<f32> {
    var z = vec2(0.0);
    for (var i = 0; i < N; i++) {
        z = cSq(z) + c;

        if i + 1 == N {
            break;
        }

        if length(z) > 4.0 {
            var value = sqrt(f32(i) / f32(N));
            return hueShift(vec3(1.0, 0.0, 0.0), value * 2.0 * PI);
        }
    }

    return vec3(0.0);
}

fn cSq(z: vec2<f32>) -> vec2<f32> {
    return vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
}

fn hueShift(color: vec3<f32>, hue: f32) -> vec3<f32> {
    var k = vec3(0.57735, 0.57735, 0.57735);
    var cosAngle = cos(hue);
    return vec3(color * cosAngle
                + cross(k, color) * sin(hue)
                + k * dot(k, color) * (1.0 - cosAngle)
    );
}
```

</details>

```rust
#[derive(ShaderType)]
struct Uniform {
    size: Vector2<u32>,
    zoom: f32,
}

const SIZE: Vector2<u32> = Vector2::new(4096, 4096);

fn main() -> Result<()> {
    let gpu = Gpu::init()?;

    let uniform = gpu.create_uniform(Uniform { size: SIZE, zoom: 0.0 })?;
    let buffer = gpu.create_storage(vec![0; (SIZE.x * SIZE.y) as usize])?;

    let pipeline = gpu
        .compute_pipeline(include_wgsl!("shader.wgsl"))
        .bind(&uniform)
        .bind(&buffer)
        .finish();

    for zoom in 0..15_00 {
        uniform.upload(Uniform {
            size: SIZE,
            zoom: zoom as f32 / 100.0,
        })?;

        pipeline.dispatch(Vector3::new(SIZE.x / 8, SIZE.y / 8, 1));
        buffer.download_async(move |result| {
            ImageBuffer::from_par_fn(SIZE.x, SIZE.y, |x, y| {
                let color = result[(y * SIZE.x + x) as usize];
                Rgb([color as u8, (color >> 8) as u8, (color >> 16) as u8])
            })
            .save(format!("rec/out-{zoom:0>4}.png"))
            .unwrap();
        });
    }

    Ok(())
}
```
