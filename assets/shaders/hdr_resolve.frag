in vec2 vs_uv;

uniform sampler2D tex;

out vec4 frag_color;

void main() {
    vec4 color = texture(tex, vs_uv);
    frag_color = color;
}


