struct Camera {
    vec4 world_xywh;
};

uniform Camera camera;
uniform vec2 screen_size;

layout (location = 0) in vec4 a_xywh;
layout (location = 1) in uint a_space;
layout (location = 2) in uint a_effect;
layout (location = 3) in vec4 a_tex_uvwh;
layout (location = 4) in vec4 a_rgba;
layout (location = 5) in uint a_tex_id;
layout (location = 6) in float a_flip;

flat out uint vs_tex_id;
flat out uint vs_effect;
out vec4 vs_rgba;
out vec2 vs_uv;
out vec2 vs_pos;

vec2 project(vec2 pos, Camera camera) {
    vec2 proj = pos;

    if (a_space == WorldSpace) {
        proj -= camera.world_xywh.xy;
    }

    if (a_space == WorldSpace || a_space == CameraSpace) {
        proj /= 0.5 * camera.world_xywh.zw;
    } else if (a_space == ScreenSpace) {
        proj /= 0.5 * screen_size;
    }

    return proj;
}

void main(void) {
    vec2 pos = a_xywh.xy;
    vec2 size = a_xywh.zw;

    pos += 0.5 * RECT_IDX_TO_NDC[gl_VertexID] * size;
    vec2 proj = project(pos, camera);

    vec2 local_uv = RECT_IDX_TO_UV[gl_VertexID];
    local_uv.y = 1.0 - local_uv.y;
    if (a_flip > 0.0) {
        local_uv.x = 1.0 - local_uv.x;
    }
    vs_uv = a_tex_uvwh.xy + local_uv * vec2(a_tex_uvwh.z, -a_tex_uvwh.w);

    vs_tex_id = a_tex_id;
    vs_effect = a_effect;
    vs_rgba = a_rgba;
    vs_pos = pos;
    gl_Position = vec4(proj, 0.0, 1.0);
}
