#version 450

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image3D img_output;

#define SCENE_SCALE 64

struct Sphere {
    vec3 position;
    float radius;
};

struct Box {
    vec3 position;
    vec3 size;
};

struct InfPlane {
    float height;
};

float dot2( in vec2 v ) { return dot(v,v); }
float dot2( in vec3 v ) { return dot(v,v); }
float ndot( in vec2 a, in vec2 b ) { return a.x*b.x - a.y*b.y; }

float sdSphere(vec3 p, float s) {
    return length(p) - s;
}

float sdPlane(vec3 p) {
    return p.y;
}

void main() {
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);
    // vec4 pixel = vec4(pixel_coords, 1.0);

    vec3 world_pos = pixel_coords;

    float mat_id = 0.0;
    float dist = sdSphere(world_pos - vec3(0.0, 0.0, -2.0), 0.5) / SCENE_SCALE;
    // float dist = sdPlane(world_pos - vec3(0.0, 0.0, 0.0)) / SCENE_SCALE;

    vec2 pixel_data = vec2(dist, mat_id);
    // vec4 pixel = vec4(pixel_data, pixel_data);
    // vec4 pixel = vec4(1.0, vec2(0.0), 1.0);
    vec4 pixel = vec4(1.0);

    imageStore(img_output, pixel_coords, pixel);
}
