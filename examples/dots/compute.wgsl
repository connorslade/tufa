@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;

struct Particle {
    position: vec2f,
    velocity: vec2f
}

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {
    particles[pos.x].position = vec2(0.0);
}
