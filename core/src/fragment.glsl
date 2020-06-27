out vec3 frag_color;

in vec3 origin;
in vec3 ray;

#define SCENE_SCALE 512

#define MAX_STEPS 1024
#define DIST_MULT 1.0

#define PI 3.14159265359
#define HALF_PI 1.570796326795
#define INV_PI 0.3183098861837697
#define TAU 6.28318530718
#define INV_TAU 0.1591549430918849

const float PHI = sqrt(5.0) * 0.5 + 0.5;

const vec3 LIGHT_DIR = normalize(vec3(-1.0, -1.0, -1.0));
const float LIGHT_INTENSITY = 1.5;
const float SUN_ANGULAR_DIAMETER = 0.5;

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
    // return texture(depth_tex, position / SCENE_SCALE, 0).x * SCENE_SCALE;
    // return texelFetch(depth_tex, ivec3(position), 0).x * SCENE_SCALE;
    // return length(position - vec3(5.0)) - 0.5;

    return texture(depth_tex, position / SCENE_SCALE, 0).x * SCENE_SCALE;

    // vec3 bias = (position - floor(position));
    // float dist = texelFetch(depth_tex, ivec3(position), 0).x * SCENE_SCALE;
    // float dist_l = texelFetch(depth_tex, ivec3(position) - ivec3(1,0,0), 0).x * SCENE_SCALE * bias.x;
    // float dist_r = texelFetch(depth_tex, ivec3(position) + ivec3(1,0,0), 0).x * SCENE_SCALE * bias.x;
    // float dist_u = texelFetch(depth_tex, ivec3(position) + ivec3(0,1,0), 0).x * SCENE_SCALE * bias.y;
    // float dist_d = texelFetch(depth_tex, ivec3(position) - ivec3(0,1,0), 0).x * SCENE_SCALE * bias.y;
    // float dist_b = texelFetch(depth_tex, ivec3(position) - ivec3(0,0,1), 0).x * SCENE_SCALE * bias.z;
    // float dist_f = texelFetch(depth_tex, ivec3(position) + ivec3(0,0,1), 0).x * SCENE_SCALE * bias.z;
    // return (dist + dist_l + dist_r + dist_u + dist_d + dist_b + dist_f) / 7.0;
}

RaycastHit castRay(vec3 origin, vec3 direction) {
    RaycastHit hit;

    float tmin = 0.02;
    float tmax = 512.0;

    float t = tmin;

    for (int i=0; i<MAX_STEPS; i++) {
        if (t>=tmax) {
            hit.colour = vec3(0.0);
            hit.dist = -1.0;
            break;
        }
        float dist = map(origin + direction * t);
        if (abs(dist) < 0.001 * t) {
            hit.colour = vec3(1.0);
            hit.dist = t;
            break;
        }
        t += dist * DIST_MULT;
    }

    return hit;
}

// http://iquilezles.org/www/articles/rmshadows/rmshadows.htm
float calcSoftshadow( in vec3 ro, in vec3 rd, in float tmin, in float tmax, float k )
{
    float res = 1.0;
    float ph = 1e20;
    for (float t=tmin; t<tmax;) {
        float h = map(ro + rd * t);
        if (h < 0.001)
            return 0.0;
        float y = h*h/(2.0*ph);
        float d = sqrt(h*h-y*y);
        res = min(res, k*d/max(0.0, t-y));
        ph = h;
        t += h;
    }
    return res;
}

// http://iquilezles.org/www/articles/normalsSDF/normalsSDF.htm
vec3 calcNormal( in vec3 pos )
{
    vec2 e = vec2(1.0, -1.0) * 0.1;
    return normalize( e.xyy*map( pos + e.xyy ) +
					  e.yyx*map( pos + e.yyx ) +
					  e.yxy*map( pos + e.yxy ) +
					  e.xxx*map( pos + e.xxx ) );
}

// https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
vec3 tonemapACES( vec3 x )
{
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return (x*(a*x+b))/(x*(c*x+d)+e);
}

vec3 get_sky(vec3 rd) {
    vec3 col = vec3(0.32, 0.36, 0.4) - rd.y * 0.4;
    float sun = clamp(dot(rd, LIGHT_DIR), 0.0, 1.0);
    col += vec3(1.0, 0.8, 0.4) * 0.2 * pow(sun, 6.0);
    col *= 2.5;
    return col;
}

////////////////////////////////////////////////////////////////////////////////
// Main function
////////////////////////////////////////////////////////////////////////////////
void main() {
    vec3 rayDir = normalize(ray);

    RaycastHit hit = castRay(origin, rayDir);

    if (hit.dist >= 0) {
        vec3 hit_pos = origin + rayDir * hit.dist;
        vec3 normal = calcNormal(hit_pos);
        float attenuation = calcSoftshadow(hit_pos, -LIGHT_DIR, 0.02, 512.0, 2.0);
        frag_color = hit.colour * dot(normal, -LIGHT_DIR) * attenuation;
        // frag_color = normal;
        // frag_color = pow(frag_color, vec3(0.4545));
    } else {
        // frag_color = vec3(0.33);
        frag_color = get_sky(normalize(rayDir));
    }

    frag_color = tonemapACES(frag_color);
}
