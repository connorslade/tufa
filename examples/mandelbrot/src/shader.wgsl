@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> data: array<u32>;

struct Uniform {
    size: vec2<u32>,
    zoom: f32
}

const PI: f32 = 3.1415926538;

const N: i32 = 1000;
const POI: vec2<f32> = vec2(-1.7864323556423187, -2.905726428359401e-7);

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {
    var zoom = 4.0 / exp(ctx.zoom);
    var c = (vec2(f32(pos.x), f32(pos.y)) / f32(ctx.size.x) - 0.5) * zoom + POI;

    var color = evaluate(c);
    var packed = pack4x8unorm(vec4(color, 0.0));

    data[ctx.size.x * pos.y + pos.x] = packed;
}

fn evaluate(c: vec2<f32>) -> vec3<f32> {
    var z = vec2(0.0);
    for (var i = 0; i < N; i++) {
        z = cSq(z) + c;
        
        if i + 1 == N { 
            break;
        }

        if length(z) > 4.0 {
            var value = sqrt(f32(i) / f32(N));
            return hueShift(vec3(1.0, 0.0, 0.0), value * 2.0 * PI);
        }
    }
    
    return vec3(0.0);
}

fn cSq(z: vec2<f32>) -> vec2<f32> {
    return vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
}

fn hueShift(color: vec3<f32>, hue: f32) -> vec3<f32> {
    var k = vec3(0.57735, 0.57735, 0.57735);
    var cosAngle = cos(hue);
    return vec3(color * cosAngle
                + cross(k, color) * sin(hue)
                + k * dot(k, color) * (1.0 - cosAngle)
    );
}
