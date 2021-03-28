#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in uint a_index;
layout(location=3) in vec2 a_tex_coord;
layout(location=4) in uint a_tex_idx;

layout(set=0, binding=0) uniform Uniforms
{
    mat4 u_view_proj;
    vec3 u_view_angle;
    vec3 u_light_dir;
    uint selected;
    uint s1;
    uint s2;
    uint s3;
};

layout(location=0) out vec3 v_position;
layout(location=1) out vec3 v_normal;
layout(location=2) out vec3 v_color;
layout(location=3) flat out uint v_index;
layout(location=4) out vec2 v_tex_coord;
layout(location=5) flat out uint v_tex_idx;

void main()
{
    v_color = vec3(0.6, 0.0, 0.8);
    if (a_index == selected) {
        v_color = vec3(0.0, 0.0, 0.0);
    } else if (a_index == s1 || a_index==s2 || a_index==s3) {
        v_color = vec3(0.0, 0.5, 0.5);
    };
    vec4 modal_space = vec4(a_position, 1.0);
    v_normal = a_normal;
    v_tex_coord = a_tex_coord;
    v_position = modal_space.xyz;
    v_index = a_index;
    v_tex_idx = a_tex_idx;
    vec3 normal = vec3(1.0, 2.0, 3.0);
    gl_Position = u_view_proj * modal_space;
}