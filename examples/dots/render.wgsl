@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> ctx: Uniform;

struct Particle {
    position: vec2f,
    velocity: vec2f
}

struct Uniform {
    window: vec2f,
    radius: f32,
    border: f32,
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

    let scale = ctx.window.yx / min(ctx.window.x, ctx.window.y);
    let position = (pos.xy * scale * ctx.radius) + (particle.position * 2.0 - 1.0);

    return VertexOutput(vec4(position, 1.0, 1.0), uv);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = 0.5 - length(in.uv - vec2(0.5));
    let border = ctx.border / ctx.radius / 1000.0;

    let inside = step(border, dist);
    let edge = step(0.0, dist) * (1.0 - inside) * (dist / border) * f32(border != 0);

    return vec4(inside + edge);
}
