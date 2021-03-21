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
};

layout(location=0) out vec4 f_color;

void main() {
    vec4 object_color = vec4(v_color, 1.0);
    float ambient_strength = 0.15;
    vec3 normal = normalize(v_normal);
    float diffuse_strength = max(dot(normal, -u_light_dir), 0.0);
    vec3 half_dir = normalize((-u_view_angle) + u_light_dir);
    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    vec3 color = vec3((ambient_strength + diffuse_strength) + specular_strength) * object_color.xyz;
    f_color = vec4(color, object_color.w);
}