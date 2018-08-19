#version 440 core
layout ( location = 0) in vec2 uv;

uniform vec2 u_field_size;
struct CellData{
    bool alive;
    float lifetime;
    float creation;
};

layout(shared, binding = 0) readonly buffer OutputData
{
    CellData next[];
};

out vec4 FragColor;

void main() {
    ivec2 xy = ivec2(int(uv.x * u_field_size.x), int(uv.y*u_field_size.y));
    int pixel_coord = xy.x + xy.y * int(u_field_size.x);
    
    CellData cell = next[pixel_coord];
    float life = cell.lifetime;

    vec3 r = vec3(mod(1*cell.creation, 2),(1+sin(cell.creation)/2),0);
    FragColor = vec4(
        r*vec3(1,1,1)*life ,
        1.0);
}