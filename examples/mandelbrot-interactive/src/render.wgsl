struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vert(
    @location(0) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.pos = pos;
    out.uv = uv;
    return out;
}

@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> data: array<u32>;

struct Uniform {
    size: vec2<u32>,
    zoom: f32
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    var pixel = vec2(u32(in.uv.x * f32(ctx.size.x)), u32(in.uv.y * f32(ctx.size.y)));
    var packed = data[pixel.y * ctx.size.x + pixel.x];
    var color = unpack4x8unorm(packed);

    return vec4(color.xyz, 1.0);
}