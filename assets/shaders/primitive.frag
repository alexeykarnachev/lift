in vec4 vs_rgba;
in vec2 vs_uv;
in vec2 vs_pos;
flat in uint vs_tex_id;
flat in uint vs_effect;

uniform vec2 light_pos;
uniform sampler2D sprite_atlas_tex;
uniform sampler2D glyph_atlas_tex;

out vec4 frag_color;

vec4 get_color() {
    vec4 color;

    if (vs_tex_id == SpriteTexture) {
        vec2 tex_size = vec2(textureSize(sprite_atlas_tex, 0));
        vec2 uv = vs_uv;
        uv = floor(uv) + min(fract(uv) / fwidth(uv), 1.0) - 0.5;
        uv /= tex_size;
        color = texture(sprite_atlas_tex, uv);
    } else if (vs_tex_id == GlyphTexture) {
        vec2 tex_size = vec2(textureSize(glyph_atlas_tex, 0));
        vec2 uv = vs_uv;
        uv /= tex_size;

        float alpha = texture(glyph_atlas_tex, uv).r;
        alpha *= vs_rgba.a;
        color = vec4(vs_rgba.rgb, alpha);
    } else {
        color = vs_rgba;
    }

    return color;
}

vec4 apply_light(vec4 color) {
    float d = distance(light_pos, vs_pos) / 40.0;
    float k = 1.0 / (1.0 + d * 0.05 + d*d * 0.1);
    vec3 rgb = color.rgb * k * vec3(1.2, 1.1, 1.0);
    return vec4(rgb, color.a);
}

void main(void) {
    vec4 color = get_color();

    if ((vs_effect & ApplyLightEffect) == 1) {
        color = apply_light(color);
    }

    frag_color = color;
}





