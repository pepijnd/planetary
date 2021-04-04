#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in uint a_index;
layout(location=3) in vec2 a_tex_coord;
layout(location=4) in uint a_tex_idx;
layout(location=5) in vec3 a_tangent;
layout(location=6) in vec3 a_bitangent;

layout(set=0, binding=0) uniform Uniforms
{
    mat4 u_view_proj;
    vec3 u_view_pos;
    vec3 u_light_pos;
    uint selected;
    uint s1;
    uint s2;
    uint s3;
};

layout(location=0) out vec3 v_position;
layout(location=1) out vec2 v_tex_coord;
layout(location=2) out vec3 v_light_pos;
layout(location=3) out vec3 v_normal;
layout(location=4) out vec3 v_view_pos;
layout(location=5) flat out uint v_index;
layout(location=6) flat out uint v_tex_idx;


void main()
{
    vec4 modal_space = vec4(a_position, 1.0);
    mat3 tangent_matrix = transpose(mat3(
        normalize(a_tangent),
        normalize(a_bitangent),
        normalize(a_normal)
    ));
    v_position = tangent_matrix * modal_space.xyz;
    v_tex_coord = a_tex_coord;
    v_light_pos = tangent_matrix * u_light_pos;
    v_normal = tangent_matrix * a_normal;
    v_view_pos = tangent_matrix * u_view_pos;
    v_index = a_index;
    v_tex_idx = a_tex_idx;
    gl_Position = u_view_proj * modal_space;
}