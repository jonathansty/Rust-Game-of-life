#version 440 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aUV;
layout (location = 2) in vec3 aColor;

layout (location = 0) out vec2 oUV;
layout (location = 1) out vec3 oColor;

void main() {
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);

    oUV = aUV;
    oColor = aColor;
}