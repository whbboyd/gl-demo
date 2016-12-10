pub static VERTEX_SHADER_SRC: &'static str = r#"
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
	"#;

pub static FRAGMENT_SHADER_SRC: &'static str = r#"
	#version 120

	uniform vec3 u_mat_specular;
	uniform vec3 u_mat_diffuse;
	uniform vec3 u_mat_ambient;

	varying vec3 v_normal;
	varying vec3 v_light;
	varying vec3 v_position;

	void main(void) {
		float diffuse = max(dot(normalize(v_normal), normalize(v_light)), 0.0);

		vec3 camera_dir = normalize(-v_position);
		vec3 half_direction = normalize(normalize(v_light) + camera_dir);
		float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 64.0);

		gl_FragColor = vec4(u_mat_ambient +
				diffuse * u_mat_diffuse +
				specular * u_mat_specular,
				1.0);
	}
	"#;

