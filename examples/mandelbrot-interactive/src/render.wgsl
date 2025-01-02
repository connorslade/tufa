struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vert(
    @location(0) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    return VertexOutput(pos, uv);
}

@group(0) @binding(0) var<uniform> ctx: Uniform;

struct Uniform {
    window: vec2<u32>,
    iters: u32,

    center: vec2<f32>,
    zoom: f32,
    power: u32
}

const PI: f32 = 3.1415926538;

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    var window = vec2<f32>(ctx.window);
    var scale = window / vec2(min(window.x, window.y));
    var pos = (in.uv - 0.5) * scale;

    var zoom = 4.0 / exp(ctx.zoom);
    var c = pos * zoom + ctx.center;

    var color = evaluate(c);
    return vec4(color.xyz, 1.0);
}

fn evaluate(c: vec2<f32>) -> vec3<f32> {
    var z = vec2(0.0);
    for (var i = u32(0); i < ctx.iters; i++) {
        z = cPow(z, ctx.power) + c;
        
        if i + 1 == ctx.iters {  break; }
        if length(z) > 4.0 {
            var value = sqrt(f32(i) / f32(ctx.iters));
            return hueShift(vec3(1.0, 0.0, 0.0), value * 2.0 * PI);
        }
    }
    
    return vec3(0.0);
}

fn cPow(z: vec2<f32>, n: u32) -> vec2<f32> {
    var arg = atan2(z.y, z.x) * f32(n);
    return pow(length(z), f32(n)) * vec2(cos(arg), sin(arg));
}

fn hueShift(color: vec3<f32>, hue: f32) -> vec3<f32> {
    var k = vec3(0.57735, 0.57735, 0.57735);
    var cosAngle = cos(hue);
    return vec3(color * cosAngle
                + cross(k, color) * sin(hue)
                + k * dot(k, color) * (1.0 - cosAngle)
    );
}
