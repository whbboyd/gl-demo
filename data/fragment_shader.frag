#version 120

uniform vec3 u_mat_specular;
uniform vec3 u_mat_diffuse;
uniform vec3 u_mat_ambient;

varying vec3 v_normal;
varying vec3 v_light;
varying vec3 v_position;

void main(void) {

	float brightness = dot(normalize(v_normal), normalize(v_light));
	vec3 matte_color = mix(u_mat_ambient, u_mat_diffuse, brightness);

	vec3 camera_dir = normalize(-v_position);
	vec3 half_direction = normalize(normalize(v_light) + camera_dir);
	float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 64.0);

	gl_FragColor = vec4(matte_color + (specular * u_mat_specular), 1.0);
}

