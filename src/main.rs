#[macro_use]
extern crate glium;
extern crate time;

// This is the hardcoded Utah Teapot model from
// https://tomaka.github.io/glium/book/tuto-07-teapot.rs
#[path = "tuto-07-teapot.rs"]
mod teapot;

use glium::{DisplayBuild, Surface};
use glium::glutin::{ElementState, Event, Window};
use time::PreciseTime;

fn main() {
	println!("Starting demo...");

	println!("Initializing display...");
	let display = glium::glutin::WindowBuilder::new()
		.with_depth_buffer(24)
		.with_vsync()
		.build_glium().unwrap();

	println!("Compiling shaders...");
	let program = glium::Program::from_source(
		&display,
		VERTEX_SHADER_SRC,
		FRAGMENT_SHADER_SRC,
		None).unwrap();

	println!("Preparing environment...");
	let params = glium::DrawParameters {
		depth: glium::Depth {
			test: glium::draw_parameters::DepthTest::IfLess,
			write: true,
			.. Default::default()
		},
		backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
		.. Default::default()
	};

	let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
	let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
	let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
												&teapot::INDICES).unwrap();
	let light = [-1.0, 0.4, 0.9f32];

	let mut frame: u64 = 0;
	let mut last_time = PreciseTime::now();

	let fps_message_interval = 500;
	let fov: f32 = 3.141592 / 3.0;

	let mut perspective = perspective_matrix(1, 1, fov);

	let mut camera = Camera {
		loc_x: 2.0,
		loc_y: 0.0,
		loc_z: 0.0,
		dir_x: -1.0,
		dir_y: 0.0,
		dir_z: 0.0
	};
	let mut movement = MovementState {
		forward: false,
		backward: false,
		left: false,
		right: false
	};

	// Main program loop
	println!("Starting program loop...");
	'main: loop {
		frame += 1;

		let mut target = display.draw();
		target.clear_color_and_depth((0.5, 0.5, 1.0, 1.0), 1.0);

		let t = (frame % 785) as f32 * 0.008;

		// Let's have the teapot spin. It's the demo version of an idle animation.
		let model = [
				[0.01 * t.cos(), 0.0,  -0.01 * t.sin(), 0.0],
				[0.0,            0.01, 0.0,             0.0],
				[0.01 * t.sin(), 0.0,  0.01 * t.cos(),  0.0],
				[0.0,            0.0,  0.0,             1.0] ];

		let view = view_matrix(&[camera.loc_x, camera.loc_y, camera.loc_z], &[camera.dir_x, camera.dir_y, camera.dir_z], &[0.0, 1.0, 0.0]);

		target.draw(
			(&positions, &normals),
			&indices,
			&program,
			&uniform! {
				model_matrix: model,
				view_matrix: view,
				perspective_matrix: perspective,
				u_light: light},
			&params).unwrap();
		target.finish().unwrap();

		// Handle events
		for ev in display.poll_events() {
			match ev {
				Event::Closed => break 'main,
				Event::Resized(w, h) =>
					perspective = perspective_matrix(w, h, fov),
				Event::KeyboardInput(ElementState::Pressed, 25, _) =>
					movement.forward = true,
				Event::KeyboardInput(ElementState::Released, 25, _) =>
					movement.forward = false,
				Event::KeyboardInput(ElementState::Pressed, 38, _) =>
					movement.left = true,
				Event::KeyboardInput(ElementState::Released, 38, _) =>
					movement.left = false,
				Event::KeyboardInput(ElementState::Pressed, 39, _) =>
					movement.backward = true,
				Event::KeyboardInput(ElementState::Released, 39, _) =>
					movement.backward = false,
				Event::KeyboardInput(ElementState::Pressed, 40, _) =>
					movement.right = true,
				Event::KeyboardInput(ElementState::Released, 40, _) =>
					movement.right = false,
				Event::MouseMoved((x, y)) =>
					update_dir(&display.get_window().unwrap(), &mut camera, x, y),
				_ => ()
			}
		}

		if movement.forward {
			camera.loc_x += camera.dir_x * 0.05;
			camera.loc_y += camera.dir_y * 0.05;
			camera.loc_z += camera.dir_z * 0.05;
		}
		if movement.backward {
			camera.loc_x -= camera.dir_x * 0.05;
			camera.loc_y -= camera.dir_y * 0.05;
			camera.loc_z -= camera.dir_z * 0.05;
		}
		if movement.left {
			camera.loc_x -= camera.dir_z * 0.05;
			camera.loc_z += camera.dir_x * 0.05;
		}
		if movement.right {
			camera.loc_x += camera.dir_z * 0.05;
			camera.loc_z -= camera.dir_x * 0.05;
		}

		// Wait for end of frame
		// We enabled vsync when creating the window, so this happens automatically.

		if frame % fps_message_interval == 0 {
			let current_time = PreciseTime::now();
			let duration = last_time.to(current_time).num_milliseconds() as f32 /
				fps_message_interval as f32;
			let fps = fps_message_interval as f32 / duration;
			last_time = current_time;
			println!("Rendered {} frames in {} seconds ({} FPS)",
				fps_message_interval,
				duration,
				fps);
		}
	}

	println!("Program loop ended, exiting...");
}

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };
    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];
    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };
    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];
    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];
    [
        [s[0], u[0], f[0], 0.0],
        [s[1], u[1], f[1], 0.0],
        [s[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

fn perspective_matrix(width: u32, height: u32, fov: f32) -> [[f32; 4]; 4] {
	let aspect_ratio = height as f32 / width as f32;

	let zfar = 1024.0;
	let znear = 0.1;

	let f = 1.0 / (fov / 2.0).tan();

	[
		[f * aspect_ratio, 0.0, 0.0,                            0.0],
		[0.0             , f,   0.0,                            0.0],
		[0.0             , 0.0, (zfar+znear)/(zfar-znear),      1.0],
		[0.0             , 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],
	]
}

fn update_dir(window: &Window, camera: &mut Camera, x: i32, y: i32) -> () {
	let (uw, uh) = window.get_inner_size().unwrap();
	let w = uw as i32;
	let h = uh as i32;
	window.set_cursor_position(w/2, h/2);
	let dx = w/2 - x;
	let dy = h/2 - y;
	if dx.abs() > 200 || dy.abs() > 200 {
		println!("Skipping camera move due to large delta: {}, {}", dx, dy);
		return;
	}

	// Turn dx into a rotation on the xz plane
	let dh = dx as f32 * 0.005;
	let new_dir_x = camera.dir_x * dh.cos() - camera.dir_z * dh.sin();
	let new_dir_z = camera.dir_x * dh.sin() + camera.dir_z * dh.cos();
	camera.dir_x = new_dir_x;
	camera.dir_z = new_dir_z;

	// Turn dy into a rotation on the xy plane
	// (not really the xy plane; it's the plane determined by xz and [0,1,0])
	// Clamp dir_y + dy
	// (the camera will flip if you cross zenith or nadir, which is super confusing)
	//FIXME: This more-or-less works, but is probably wrong.
	camera.dir_y += dy as f32 * 0.005;

	let norm_factor = camera.dir_x.abs() + camera.dir_y.abs() + camera.dir_z.abs();
	camera.dir_x /= norm_factor;
	camera.dir_y /= norm_factor;
	camera.dir_z /= norm_factor;
}

#[derive(Debug)]
struct Camera {
	loc_x: f32,
	loc_y: f32,
	loc_z: f32,
	dir_x: f32,
	dir_y: f32,
	dir_z: f32
}

struct MovementState {
	forward: bool,
	backward: bool,
	left: bool,
	right: bool
}

static VERTEX_SHADER_SRC: &'static str = r#"
	#version 120

	attribute vec3 position;
	attribute vec3 normal;

	uniform mat4 model_matrix;
	uniform mat4 view_matrix;
	uniform mat4 perspective_matrix;

	varying vec3 v_normal;

		void main() {
			mat4 model_view_matrix = view_matrix * model_matrix;
			v_normal = transpose(inverse(mat3(model_view_matrix))) * normal;
			gl_Position = perspective_matrix * model_view_matrix * vec4(position, 1.0);
		}
	"#;

static FRAGMENT_SHADER_SRC: &'static str = r#"
		#version 120

		varying vec3 v_normal;
		uniform vec3 u_light;

		void main(void) {
			float brightness = dot(normalize(v_normal), normalize(u_light));
			vec3 dark_color = vec3(0.0, 0.2, 0.0);
			vec3 regular_color = vec3(0.1, 0.7, 0.1);
			gl_FragColor = vec4(mix(dark_color, regular_color, brightness), 1.0);
		}
	"#;
