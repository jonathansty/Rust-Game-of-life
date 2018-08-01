#version 440 core
layout ( location = 0) in vec2 uv;
layout ( location = 1) in vec3 color;

uniform float u_time;

// Prev state 
uniform sampler2D u_texture0;

out vec4 FragColor;

void main() {
    // FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    vec4 tex0 = textureOffset(u_texture0, uv, ivec2(1,0));
    vec4 tex1 = textureOffset(u_texture0, uv, ivec2(0,1));
    vec4 tex2 = textureOffset(u_texture0, uv, ivec2(-1,0));
    vec4 tex3 = textureOffset(u_texture0, uv, ivec2(0,-1));
    vec4 result = tex0 + tex1 + tex2 + tex3;
    // FragColor = vec4(result.rgb / 4.0f, 1.0f);
    FragColor = texture(u_texture0, uv);
}