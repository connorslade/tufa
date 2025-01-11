@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> ctx: Uniform;

struct Uniform {
    window: vec2f,
    radius: f32,
    border: f32,
    speed: f32
}

struct Particle {
    position: vec2f,
    velocity: vec2f
}

@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let particle = particles[id.x];

    var pos = particle.position + particle.velocity * ctx.speed;

    if pos.x > 1.0 { pos.x = 0.0; }
    else if pos.x < 0.0 { pos.x = 1.0; }

    if pos.y > 1.0 { pos.y = 0.0; }
    else if pos.y < 0.0 { pos.y = 1.0; }

    particles[id.x].position = pos;
}
