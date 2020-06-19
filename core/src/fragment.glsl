out vec3 frag_color;

in vec3 origin;
in vec3 ray;

#define MAX_STEPS 128
#define SCENE_SCALE 64

struct Material {
    vec3 albedo;
    float roughness;
    float metalness;
};

struct RaycastHit {
    float dist;
    int mat_id;
};

// uniform Material materials[32];

uniform sampler3D depth_tex;

float getDistance(vec3 position) {
    return texture(depth_tex, position / SCENE_SCALE).x;
}

vec3 castRay(vec3 origin, vec3 direction) {
    vec3 result = vec3(0.0);

    float tmin = 0.02;
    float tmax = 512.0;

    float t = tmin;

    for (int i=0; i<MAX_STEPS; i++) {
        if (t>=tmax) { break; }
        float dist = getDistance(origin + direction * t);
        if (abs(dist) < 0.001 * t) {
            result = vec3(dist);
            break;
        }
        t += dist;
    }

    return result;
}

void main() {
    vec3 rayDir = normalize(ray);

    frag_color = castRay(origin, rayDir);
    float dist = texture(depth_tex, vec3(0.0)).x;
    frag_color = vec3(dist);
}
