@group(0)
@binding(0)
var<storage, read_write> data: Data;

struct Data {
    a: f32,
    b: f32,
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    data.a = data.a + data.b;
}
