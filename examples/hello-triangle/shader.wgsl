@group(0) @binding(0) var<uniform> ctx: Uniform;

struct Uniform {
    transform: mat4x4f
}

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(1) uv: vec2f,
}

@vertex
fn vert(
    @location(0) pos: vec4f,
    @location(1) uv: vec2f
) -> VertexOutput {
    return VertexOutput(ctx.transform * pos, uv);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let color = vec3(1.0, 0.0, 0.0) * (1.0 - in.uv.x - in.uv.y) + vec3(0.0, in.uv);
    return vec4(color, 1.0);
}
