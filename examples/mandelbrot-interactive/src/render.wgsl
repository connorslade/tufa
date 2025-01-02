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
    center: vec2<f32>,
    iters: u32,
    zoom: f32,
}

const PI: f32 = 3.1415926538;

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    var window = vec2f(ctx.window);
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
        z = cSq(z) + c;
        
        if i + 1 == ctx.iters {  break; }
        if length(z) > 4.0 {
            var value = sqrt(f32(i) / f32(ctx.iters));
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
