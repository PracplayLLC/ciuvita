#version 450

layout(location = 0) out vec2 v_TexCoord;

void main() {
    vec2 tc = vec2(0.0);
    switch(gl_VertexIndex) {
        case 0: tc = vec2(1.0, 0.0); break;
        case 1: tc = vec2(1.0, 1.0); break;
        case 2: tc = vec2(0.0, 0.0); break;
        case 3: tc = vec2(0.0, 1.0); break;
        default: break;
    }
    v_TexCoord = tc;
    vec2 pos = tc * 2.0 - 1.0;
    gl_Position = vec4(pos.x, -pos.y, 0.5, 1.0);
}