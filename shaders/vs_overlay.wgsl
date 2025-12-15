struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct Overlay {
    position: vec4<f32>,
    size: vec4<f32>,
    color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> overlay: Overlay;

@vertex
fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(in.position.x * overlay.size.x, in.position.y * overlay.size.y, in.position.z, 1.0) + overlay.position.xyzw;
    out.color = vec4<f32>(overlay.color.xyz * overlay.color.w, overlay.color.w);

    return out;
}
