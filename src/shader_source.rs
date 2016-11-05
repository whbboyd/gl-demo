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

		void main() {
			mat4 model_view_matrix = view_matrix * model_matrix;
			v_normal = transpose(inverse(mat3(model_view_matrix))) * normal;
			v_light = transpose(inverse(mat3(view_matrix))) * u_light;
			gl_Position = perspective_matrix * model_view_matrix * vec4(position, 1.0);
		}
	"#;

pub static FRAGMENT_SHADER_SRC: &'static str = r#"
		#version 120

		uniform vec3 u_mat_light;
		uniform vec3 u_mat_dark;

		varying vec3 v_normal;
		varying vec3 v_light;

		void main(void) {
			float brightness = dot(normalize(v_normal), normalize(v_light));
			gl_FragColor = vec4(mix(u_mat_dark, u_mat_light, brightness), 1.0);
		}
	"#;

