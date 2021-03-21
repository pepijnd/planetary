#version 450

layout(location=0) in vec3 v_position;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec3 v_color;
layout(location=3) flat in uint v_index;

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

layout(location=0) out uint f_index;

void main() {
    f_index = v_index;
}