struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(1) bary: vec3f
}

@vertex
fn vert(
    @location(0) pos: vec4f,
    @location(1) bary: vec3f
) -> VertexOutput {
    return VertexOutput(pos, bary);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    return vec4(vec3(select(0.0, 1.0, in.bary.x * in.bary.y - in.bary.z > 0.0)), 1.0);
}
