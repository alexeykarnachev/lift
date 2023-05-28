in vec4 vs_rgba;
in vec2 vs_uv;
in vec2 vs_pos;
flat in uint vs_tex_id;
flat in uint vs_effect;

struct Light {
    vec2 position;
    vec3 color;
    vec3 attenuation;
};

uniform Light lights[32];
uniform int n_lights;
uniform sampler2D sprite_atlas_tex;
uniform sampler2D glyph_atlas_tex;

out vec4 frag_color;

vec2 random2(vec2 p) {
    return fract(sin(vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)))) * 43758.5453);
}

vec4 get_color() {
    vec4 color;

    if (vs_tex_id == SpriteTexture) {
        vec2 tex_size = vec2(textureSize(sprite_atlas_tex, 0));
        vec2 uv = vs_uv;
        uv = floor(uv) + min(fract(uv) / fwidth(uv), 1.0) - 0.5;
        uv /= tex_size;
        color = texture(sprite_atlas_tex, uv);
        color = vec4(color.rgb + vs_rgba.rgb * vs_rgba.a, color.a);
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
    vec3 rgb = vec3(0.0);
    vec2 pos = floor(vs_pos); 
    for (int i = 0; i < n_lights; ++i) {
        Light light = lights[i];
        float d = distance(light.position, pos);
        float k = 1.0 / dot(light.attenuation, vec3(1.0, d, d * d));

        rgb += color.rgb * k * light.color;
    }

    return vec4(rgb, color.a);
}

vec4 apply_stone_wall(vec4 color) {
    float squareness = 0.2;
    vec2 scale = vec2(1.0, 1.5) * 0.02;
    vec2 pos = floor(vs_pos); 
    vec2 tile = pos * scale;
    vec2 itile = floor(tile);

    float dist0 = 1.0;
    float dist1 = 1.0;
    for (int y = -1; y <= 1; ++y) {
        for (int x = -1; x <= 1; ++x) {
            vec2 neighbor = vec2(x, y);
            vec2 rnd = random2(itile + neighbor);
            vec2 center = neighbor + itile + (1.0 - squareness) * rnd;
            vec2 diff = tile - center;
            float dist = length(diff);
            if (dist < dist0) {
                dist1 = dist0;
                dist0 = dist;
            } else if (dist < dist1) {
                dist1 = dist;
            }
        }
    }

    float edge = dist1 - dist0;
    vec3 c = color.rgb * edge;
    c = floor(c * 8.0) / 8.0;
    return vec4(c * 0.2, color.a);
}

void main(void) {
    vec4 color = get_color();

    if ((vs_effect & StoneWallEffect) > 0) {
        color = apply_stone_wall(color);
    }

    if ((vs_effect & ApplyLightEffect) > 0) {
        color = apply_light(color);
    }

    if ((vs_effect & AlphaEffect01) > 0) {
        color.a *= 0.1;
    }

    if ((vs_effect & AlphaEffect02) > 0) {
        color.a *= 0.2;
    }

    if ((vs_effect & AlphaEffect03) > 0) {
        color.a *= 0.3;
    }

    if ((vs_effect & AlphaEffect05) > 0) {
        color.a *= 0.5;
    }

    if ((vs_effect & AlphaEffect07) > 0) {
        color.a *= 0.7;
    }

    if ((vs_effect & AlphaEffect08) > 0) {
        color.a *= 0.8;
    }

    if ((vs_effect & AlphaEffect09) > 0) {
        color.a *= 0.9;
    }

    frag_color = color;
}





