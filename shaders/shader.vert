#version 450

layout(location = 0) out vec2 texCoord;

void main() {
    vec2 pos;

    pos.x = float(1 - int(gl_VertexIndex)) * 0.5;
    pos.y = float(int(gl_VertexIndex & 1) * 2 - 1) * 0.5;

    gl_Position = vec4(pos, 0.1, 1.0);
}
