#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;

// Input uniform values
uniform sampler2D texture0;
uniform vec4 colDiffuse;

uniform vec4 colorA;
uniform vec4 colorB;

// Output fragment color
out vec4 finalColor;

const float radius = 20.0;
const float rad_thresh = 1.0;
//const vec4 colorA = vec4(0.0, 1.0, 0.5, 1.0);
//const vec4 colorB = vec4(0.5, 0.3, 0.9, 1.0);

float random(vec2 q)
{
    return fract(sin(dot(q, vec2(1.6123,78.4563)))*43534.1231);
}

float noise(vec2 st)
{
    vec2 i = floor(st);
    vec2 f = fract(st);

    float a = random(i);
    float b = random(i + vec2(1.,0.));
    float c = random(i + vec2(0., 1.));
    float d = random(i + vec2(1., 1.));

    vec2 u = f*f;

    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

float value_noise(vec2 st)
{
    float f = 0.;
    float a = 0.5;
    mat2 rot = mat2(vec2(1.2, 1.6), vec2(1.6, -1.2));
    f += a*noise(st); st *= rot; a *= 0.5; st *= 1.5;
    f += a*noise(st); st *= rot; a *= 0.5; st *= 1.5;
    return f;
}

vec2 rotate(vec2 uv, float angle)
{
    mat2 rotation = mat2(vec2(sin(angle), -cos(angle)),
                        vec2(cos(angle), sin(angle)));

    uv = uv * rotation;
    return uv;
}

const float PI = 3.141593;

const int blur = 1;

void main()
{
    vec2 st = (fragTexCoord-0.5);
    float rot_angle = PI*1.618;
    float r = length(st)*2.;
    float angle = atan(st.x, st.y)/(2.*PI);
    float a = float(r < rad_thresh);
    
    float f = 0.;
    f = value_noise(vec2(abs(angle)*60., r*radius+colorA.g));

	//sigmoid
	f = 1.0/(1.0+pow(2.2718, -f));
    
    vec3 color = mix(colorA, colorB, f).rgb;
	if (abs(r-rad_thresh) < 0.005) {
		color = vec3(0.0);
	}
	finalColor = vec4(color, a)*colDiffuse;
}
