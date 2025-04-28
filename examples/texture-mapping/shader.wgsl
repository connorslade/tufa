@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct Uniform {
    transform: mat4x4f,
    flags: u32
}

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(1) uv: vec2f,
    @location(2) @interpolate(linear) linear_uv: vec2f,
}

@vertex
fn vert(
    @location(0) pos: vec4f,
    @location(1) uv: vec2f
) -> VertexOutput {
    return VertexOutput(ctx.transform * pos, uv, uv);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    if ctx.flags == 0 {
        return textureSample(texture, texture_sampler, in.linear_uv);
    } else {
        return textureSample(texture, texture_sampler, in.uv);
    }
}
