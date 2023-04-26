struct Camera {
    vec4 xywh;
    float orientation;
};

uniform Camera camera;

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

vec2 world2proj(vec2 world_pos, Camera camera) {
    vec2 half_size = vec2(0.5 * camera.xywh.zw);

    vec2 view_pos = rotate2d(world_pos, vec2(0.0), -camera.orientation);
    view_pos -= camera.xywh.xy;

    return view_pos / half_size;
}

void main(void) {
    vec2 size = a_xywh.zw;
    vec2 pos = a_xywh.xy;

    vec2 proj_pos = pos + 0.5 * RECT_IDX_TO_NDC[gl_VertexID] * size;
    proj_pos = rotate2d(proj_pos, pos, a_orientation);
    proj_pos = world2proj(proj_pos, camera);

    vs_rgba = a_rgba;
    gl_Position = vec4(proj_pos, 0.0, 1.0);
}
