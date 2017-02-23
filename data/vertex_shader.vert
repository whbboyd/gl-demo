#version 120

attribute vec3 position;
attribute vec3 normal;
attribute vec2 tex_uv;

uniform mat4 model_view_perspective_matrix;
uniform mat3 normal_matrix;
uniform mat3 light_matrix;
uniform vec3 u_light_pos;

varying vec3 v_position;
varying vec3 v_normal;
varying vec2 v_tex_uv;
varying vec3 v_light_pos;

void main() {
	v_position = vec3(model_view_perspective_matrix * vec4(position, 1.0));
	v_normal = normal_matrix * normal;
	v_tex_uv = tex_uv;
	v_light_pos = light_matrix * u_light_pos;
	gl_Position = model_view_perspective_matrix * vec4(position, 1.0);
}

