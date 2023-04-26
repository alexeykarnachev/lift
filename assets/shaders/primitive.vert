layout (location = 0) in vec4 a_xywh;
layout (location = 1) in vec4 a_rgba;
layout (location = 2) in float a_orientation;


out vec4 vs_rgba;

vec2 rotate2d(vec2 point, vec2 center, float angle) {
    vec2 p0 = point - center;
    float c = cos(angle);
    float s = sin(angle);
    vec2 p1 = vec2(p0.x * c - p0.y * s, p0.y * c + p0.x * s);
    p1 += center;
    return p1;
}

void main(void) {
    vec2 size = a_xywh.zw;
    vec2 pos = a_xywh.xy;
    vec2 ndc = pos + 0.5 * RECT_IDX_TO_NDC[gl_VertexID] * size;
    ndc = rotate2d(ndc, pos, a_orientation);

    vs_rgba = a_rgba;
    gl_Position = vec4(ndc, 0.0, 1.0);
}
