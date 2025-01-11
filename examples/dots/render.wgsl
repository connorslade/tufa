@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;

struct Particle {
    position: vec2f,
    velocity: vec2f
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>
};

@vertex
fn vert(
    @builtin(instance_index) index: u32,
    @location(0) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    let particle = particles[index];
    return VertexOutput(
        pos + vec4(particle.position * 2.0 - 1.0, 0.0, 0.0),
        uv,
    );
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius = 0.01;
    let border = 0.001;

    let dist = radius - length(in.uv - vec2(0.5));

    if dist > border {
        return vec4(1.0);
    } else if dist > 0.0 {
        return vec4(dist / border);
    }

    return vec4(0.0);
}
