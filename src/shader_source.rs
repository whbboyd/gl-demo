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

		varying vec3 v_normal;
		varying vec3 v_light;

		void main(void) {
			float brightness = dot(normalize(v_normal), normalize(v_light));
			vec3 dark_color = vec3(0.0, 0.2, 0.0);
			vec3 regular_color = vec3(0.1, 0.7, 0.1);
			gl_FragColor = vec4(mix(dark_color, regular_color, brightness), 1.0);
		}
	"#;

