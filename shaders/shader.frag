#version 450

layout(pixel_center_integer) in vec4 gl_FragCoord;

layout(location = 0) out vec4 fragColor;

void main() {
    fragColor = vec4(gl_FragCoord.xyz, 0.125);
}
