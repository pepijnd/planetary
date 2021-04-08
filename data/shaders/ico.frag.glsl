#version 450

layout(location=0) in vec3 v_position;
layout(location=1) in vec2 v_tex_coord;
layout(location=2) in vec3 v_light_pos;
layout(location=3) in vec3 v_normal;
layout(location=4) in vec3 v_view_pos;
layout(location=5) flat in uint v_index;
layout(location=6) flat in uint v_tex_idx;


layout(set=0, binding=0) uniform Uniforms
{
    mat4 u_view_proj;
    vec3 u_view_pos;
    vec3 u_light_pos;
    uint selected;
};

layout(set=1, binding=0) uniform texture2DArray t_diffuse;
layout(set=1, binding=1) uniform sampler s_diffuse;

layout(set=2, binding=0) uniform texture2D t_normal;
layout(set=2, binding=1) uniform sampler s_normal;

layout(location=0) out vec4 f_color;

void main() {
    vec4 object_color = texture(sampler2DArray(t_diffuse, s_diffuse), vec3(v_tex_coord, float(v_tex_idx)));
    vec4 object_normal = texture(sampler2D(t_normal, s_normal), v_tex_coord);

    float ambient_strength = 0.05;

    vec3 normal = normalize(object_normal.rgb * 2.0 - 1.0);
    vec3 light_dir = normalize(v_light_pos - v_position);

    float diffuse_strength = max(dot(normal, light_dir), 0.0);
    vec3 view_dir = normalize(v_view_pos - v_position);
    vec3 half_dir = normalize(view_dir + light_dir);

    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0) * 0.25;
    vec3 color = vec3(ambient_strength + diffuse_strength + specular_strength) * object_color.xyz;
    f_color = vec4(color, object_color.w);
}