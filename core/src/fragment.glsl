out vec3 frag_color;

in vec3 origin;
in vec3 ray;

#define SCENE_SCALE 128

#define MAX_STEPS 128
#define DIST_MULT 1.0

const vec3 LIGHT_DIR = normalize(vec3(-1.0, -1.0, -1.0));

struct Material {
    vec3 albedo;
    float roughness;
    float metalness;
};

struct RaycastHit {
    float dist;
    // int mat_id;
    vec3 colour;
};

// uniform Material materials[32];

uniform sampler3D depth_tex;

float map(vec3 position) {
    return texture(depth_tex, position / SCENE_SCALE).x * SCENE_SCALE;
    // return length(position - vec3(5.0)) - 0.5;
}

RaycastHit castRay(vec3 origin, vec3 direction) {
    // vec3 result = vec3(0.0);
    RaycastHit hit;

    float tmin = 0.02;
    float tmax = 512.0;

    float t = tmin;

    for (int i=0; i<MAX_STEPS; i++) {
        if (t>=tmax) { break; }
        float dist = map(origin + direction * t);
        if (abs(dist) < 0.01 * t) {
            hit.colour = vec3(1.0, t / SCENE_SCALE, 0.0);
            hit.dist = t;
            break;
        }
        t += dist * DIST_MULT;
    }

    return hit;
}

// http://iquilezles.org/www/articles/normalsSDF/normalsSDF.htm
vec3 calcNormal( in vec3 pos )
{
    vec2 e = vec2(1.0, -1.0) * 0.01;
    return normalize( e.xyy*map( pos + e.xyy ) +
					  e.yyx*map( pos + e.yyx ) +
					  e.yxy*map( pos + e.yxy ) +
					  e.xxx*map( pos + e.xxx ) );
}

void main() {
    vec3 rayDir = normalize(ray);

    RaycastHit hit = castRay(origin, rayDir);
    vec3 normal = calcNormal(origin + rayDir * hit.dist);
    frag_color = hit.colour * dot(normal, -LIGHT_DIR);
    // frag_color = normal;
}
