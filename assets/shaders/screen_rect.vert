out vec2 vs_uv;


void main() {
    vs_uv = RECT_IDX_TO_UV[gl_VertexID];
    gl_Position = vec4(RECT_IDX_TO_NDC[gl_VertexID], 0.0, 1.0);
}


