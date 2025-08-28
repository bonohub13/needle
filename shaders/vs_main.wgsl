struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> background: vec4<f32>;

@vertex
fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(in.position, 1.0);
    out.color = background.xyzw;

    return out;
}
