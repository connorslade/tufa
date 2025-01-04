@group(0)
@binding(0)
var<storage, read_write> data: Data;

struct Data {
    out: f32,
    items: array<f32>
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    for (var i = u32(0); i < arrayLength(&data.items); i++) {
        data.out += data.items[i];
    }
}