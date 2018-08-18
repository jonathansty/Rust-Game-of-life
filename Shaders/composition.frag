#version 440 core
layout ( location = 0) in vec2 uv;

uniform vec2 u_field_size;
struct CellData{
    float lifetime;
    bool alive;
};

layout(std430, binding = 0) readonly buffer OutputData
{
    CellData next[];
};

out vec4 FragColor;

void main() {
    ivec2 xy = ivec2(int(uv.x * u_field_size.x), int(uv.y*u_field_size.y));
    int pixel_coord = xy.x + xy.y * int(u_field_size.x);
    
    CellData cell = next[pixel_coord];
    float life = cell.lifetime;

    FragColor = mix(vec4(0,0,0,1), vec4(1.0,1.0,1.0,1), cell.alive?1.0:0.0);
    // vec3 c = sin(vec3(4,1,1)*(life)) / 20.0;
    FragColor = vec4(
        vec3(1,1,1)*life ,
        1.0);
}