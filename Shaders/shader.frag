#version 440 core
layout ( location = 0) in vec2 uv;
layout ( location = 1) in vec3 color;

out vec4 FragColor;

void main() {
    // FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    FragColor = vec4(color.rgb,1.0f);
}