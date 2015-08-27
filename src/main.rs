#[macro_use]
extern crate glium;
use glium::DisplayBuild;
use glium::Surface;
use glium::glutin::Event;

fn main() {
	println!("Starting demo...");

	println!("Initializing display...");
	let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
	println!("Compiling shaders...");
	let program = glium::Program::from_source(
		&display,
		VERTEX_SHADER_SRC,
		FRAGMENT_SHADER_SRC,
		None).unwrap();
	let shape = vec![
		Vertex{coords:[-0.5, -0.5]},
		Vertex{coords:[0.0, 0.5]},
		Vertex{coords:[0.5, -0.5]}];
	let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
	let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

	// Main program loop
	println!("Starting program loop...");
	'main: loop {
		let mut target = display.draw();
		target.clear_color(0.5, 0.5, 1.0, 1.0);
		target.draw(
			&vertex_buffer,
			&indices,
			&program,
			&glium::uniforms::EmptyUniforms,
			&Default::default()).unwrap();
		target.finish().unwrap();

		// Handle events
		for ev in display.poll_events() {
			println!("Got event: {:?}", ev);
			match ev {
				Event::Closed => break 'main,
				_ => ()
			}
		}

		// Wait for end of frame
	}

	println!("Program loop ended, exiting...");
}

#[derive(Copy, Clone)]
struct Vertex {
	coords: [f32; 2]
}
implement_vertex!(Vertex, coords);

static VERTEX_SHADER_SRC: &'static str = r#"
	#version 140

	in vec2 position;

		void main() {
			gl_Position = vec4(position, 0.0, 1.0);
		}
	"#;

static FRAGMENT_SHADER_SRC: &'static str = r#"
	#version 140

	out vec4 color;

	void main() {
		color = vec4(0.2, 0.8, 0.2, 1.0);
	}
"#;

