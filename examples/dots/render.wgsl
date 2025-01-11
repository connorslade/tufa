struct Dot {
    @location(2) position: vec2f,
    @location(3) radius: f32
}

struct VertexInput {
    @location(0) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vert(
    vertex: VertexInput,
    instance: Dot
) -> VertexOutput {
    return VertexOutput(
        vertex.pos + vec4(instance.position, 0.0, 0.0),
        vertex.uv
    );
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    if length(in.uv - vec2(0.5)) <= 0.05 {
        return vec4(1.0);
    }
    return vec4(0.0);
}
