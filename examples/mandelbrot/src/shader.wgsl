@group(0)
@binding(0)
var<storage, read_write> data: Data;

struct Data {
    image: array<vec3<f32>, 16384>
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    data.image[100 * global_id.y + global_id.x] = vec3(1.0);
}