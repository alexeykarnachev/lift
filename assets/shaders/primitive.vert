struct Camera {
    vec4 world_xywh;
    float orientation;
};

uniform Camera camera;

layout (location = 0) in vec4 a_world_xywh;
layout (location = 1) in vec4 a_tex_uvwh;
layout (location = 2) in vec4 a_rgba;
layout (location = 3) in uint a_tex_id;
layout (location = 4) in float a_orientation;
layout (location = 5) in float a_flip;

flat out uint vs_tex_id;
out vec4 vs_rgba;
out vec2 vs_uv;

vec2 rotate2d(vec2 point, vec2 center, float angle) {
    vec2 p0 = point - center;
    float c = cos(angle);
    float s = sin(angle);
    vec2 p1 = vec2(p0.x * c - p0.y * s, p0.y * c + p0.x * s);
    p1 += center;
    return p1;
}

vec2 world2proj(vec2 world_pos, Camera camera) {
    vec2 half_size = vec2(0.5 * camera.world_xywh.zw);

    vec2 view_pos = rotate2d(world_pos, camera.world_xywh.xy, -camera.orientation);
    view_pos -= camera.world_xywh.xy;

    return view_pos / half_size;
}

void main(void) {
    vec2 pos = a_world_xywh.xy;
    vec2 size = a_world_xywh.zw;

    vec2 proj_pos = pos + 0.5 * RECT_IDX_TO_NDC[gl_VertexID] * size;
    proj_pos = rotate2d(proj_pos, pos, a_orientation);
    proj_pos = world2proj(proj_pos, camera);

    vec2 local_uv = RECT_IDX_TO_UV[gl_VertexID];
    local_uv.y = 1.0 - local_uv.y;
    if (a_flip > 0.0) {
        local_uv.x = 1.0 - local_uv.x;
    }
    vs_uv = a_tex_uvwh.xy + local_uv * vec2(a_tex_uvwh.z, -a_tex_uvwh.w);

    vs_tex_id = a_tex_id;
    vs_rgba = a_rgba;
    gl_Position = vec4(proj_pos, 0.0, 1.0);
}
