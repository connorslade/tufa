@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> ctx: Uniform;

struct Particle {
    position: vec2f,
    velocity: vec2f
}

struct Uniform {
    window: vec2f,
    radius: f32,
    speed: f32
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
    let border = 0.001;

    let scale = ctx.window / min(ctx.window.x, ctx.window.y);
    let dist = ctx.radius - length((in.uv - vec2(0.5)) * scale);

    if dist > border {
        return vec4(1.0);
    } else if dist > 0.0 {
        return vec4(dist / border);
    }

    return vec4(0.0);
}
