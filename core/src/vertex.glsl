in vec3 v_pos;
in vec2 v_uv;

uniform mat4 inv_projview_matrix;

#define FAR 512.0
#define NEAR 0.02

out vec3 origin;
out vec3 ray;

void main() {
    // vec2 pos = (f_uv - 0.5) * 2.0; //Remap from [0,1] to [-1,-1]
    vec2 pos = v_pos.xy;
    origin = (inv_projview_matrix * vec4(pos, -1.0, 1.0) * NEAR).xyz;
    ray = (inv_projview_matrix * vec4(pos * (FAR - NEAR), FAR + NEAR, FAR - NEAR)).xyz;

    gl_Position = vec4(v_pos, 1.0);
}
