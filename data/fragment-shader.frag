#version 120

uniform vec3 u_mat_specular;
uniform vec3 u_mat_ambient;
uniform vec3 u_light_color;
uniform sampler2D u_mat_texture;

varying vec3 v_position;
varying vec3 v_normal;
varying vec2 v_tex_uv;
varying vec3 v_light_pos;

void main(void) {

	float brightness = dot(normalize(v_normal), normalize(v_light_pos));

	vec3 tex_color = texture2D(u_mat_texture, v_tex_uv).xyz;
	vec3 matte_color = mix(u_mat_ambient * tex_color,
	                       u_light_color * tex_color,
	                       brightness);

	vec3 camera_dir = normalize(-v_position);
	vec3 half_direction = normalize(normalize(v_light_pos) + camera_dir);
	float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 64.0);

	gl_FragColor = vec4(matte_color + (specular * u_mat_specular), 1.0);
}

