#version 120

attribute vec3 position;
attribute vec3 normal;

uniform mat4 model_matrix;
uniform mat4 view_matrix;
uniform mat4 perspective_matrix;
uniform vec3 u_light;

varying vec3 v_normal;
varying vec3 v_light;
varying vec3 v_position;

void main() {
	mat4 model_view_matrix = view_matrix * model_matrix;
	v_normal = transpose(inverse(mat3(model_view_matrix))) * normal;
	v_light = transpose(inverse(mat3(view_matrix))) * u_light;
	v_position = vec3(perspective_matrix * model_view_matrix * vec4(position, 1.0));
	gl_Position = perspective_matrix * model_view_matrix * vec4(position, 1.0);
}

