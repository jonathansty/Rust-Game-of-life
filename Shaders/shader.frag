#version 440 core
layout ( location = 0) in vec2 uv;
layout ( location = 1) in vec3 color;

uniform float u_time;

// Prev state 
uniform sampler2D u_texture0;

out vec4 FragColor;

void main() {
    // FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    // vec4 texture_color = texture(u_texture0, uv);

    vec4 texel0 = textureOffset(u_texture0, uv, ivec2(-1,0));
    vec4 texel1 = textureOffset(u_texture0, uv, ivec2(0,1));
    vec4 texel2 = textureOffset(u_texture0, uv, ivec2(1,0));
    vec4 texel3 = textureOffset(u_texture0, uv, ivec2(0,-1));

    vec4 blur = texel0 + texel1 + texel2 + texel3;
    blur /= 4;
    FragColor = blur;
}