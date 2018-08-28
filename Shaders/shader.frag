#version 440 core
layout ( location = 0) in vec2 uv;
layout ( location = 1) in vec3 color;

uniform sampler2D u_texture; // Previous state it's texture we can sample from

out vec4 FragColor;

void main() {
    float curr = texture(u_texture,uv).r;
    bool alive = curr > 0;

    int count = 0;
    for(int i = -1; i <= 1; ++i)
    {
        for(int j = -1; j <= 1; ++j)
        {
            if(i == 0 && j == 0)
             continue;

            float tex = textureOffset(u_texture,uv,ivec2(i,j)).r;
            if(tex > 0)
                ++count;
        }
    }

    float new_cell = curr;
    if(count < 2)                                   new_cell = 0.0f;
    else if(alive && (count == 2 || count == 3))    new_cell = 1.0f;
    else if(alive && count > 3)                     new_cell = 0.0f;
    else if(!alive && count == 3)                   new_cell = 1.0f;

    FragColor =  vec4(new_cell,new_cell,new_cell,1.0f);
}