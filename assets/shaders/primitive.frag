in vec4 vs_rgba;
in vec2 vs_uv;
flat in uint vs_use_tex;

uniform sampler2D sprite_atlas_tex;
uniform sampler2D glyph_atlas_tex;

out vec4 frag_color;

void main(void) {
    if (vs_use_tex == 1) {
        vec2 tex_size = vec2(textureSize(sprite_atlas_tex, 0));
        vec2 uv = vs_uv;
        uv = floor(uv) + min(fract(uv) / fwidth(uv), 1.0) - 0.5;
        uv /= tex_size;

        vec4 color = texture(sprite_atlas_tex, uv);

        frag_color = color;
    } else {
        frag_color = vs_rgba; 
    }
}





