#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;
layout(location=2) in vec3 a_normal;
layout(location=3) in vec3 m_position;

layout(set=1, binding=0) uniform Uniforms
{
    mat4 u_view_proj;
    vec3 u_view_angle;
    vec3 u_light_dir;
    float u_light_mix;
};

layout(location=0) out vec3 v_position;
layout(location=1) out vec2 v_tex_coords;
layout(location=2) out vec3 v_normal;

void main()
{
    v_tex_coords = a_tex_coords;
    v_normal = a_normal;
    vec4 modal_space = vec4(a_position + m_position, 1.0);
    v_position = modal_space.xyz;
    gl_Position = u_view_proj * modal_space;
}

