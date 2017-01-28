#version 120

attribute vec3 position;
attribute vec3 normal;
attribute vec2 tex_uv;

uniform mat4 model_matrix;
uniform mat4 view_matrix;
uniform mat4 perspective_matrix;
uniform vec3 u_light_pos;

varying vec3 v_position;
varying vec3 v_normal;
varying vec2 v_tex_uv;
varying vec3 v_light_pos;

void main() {
	mat4 model_view_matrix = view_matrix * model_matrix;
	v_position = vec3(perspective_matrix * model_view_matrix * vec4(position, 1.0));
	v_normal = transpose(inverse(mat3(model_view_matrix))) * normal;
	v_tex_uv = tex_uv;
	v_light_pos = transpose(inverse(mat3(view_matrix))) * u_light_pos;
	gl_Position = perspective_matrix * model_view_matrix * vec4(position, 1.0);
}

