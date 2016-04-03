#[macro_use]
extern crate glium;
extern crate time;

// This is the hardcoded Utah Teapot model from
// https://tomaka.github.io/glium/book/tuto-07-teapot.rs
#[path = "tuto-07-teapot.rs"]
mod teapot;

mod display_math;
mod shader_source;

use glium::{DisplayBuild, Surface};
use glium::glutin::{ElementState, Event};
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
		shader_source::VERTEX_SHADER_SRC,
		shader_source::FRAGMENT_SHADER_SRC,
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
	let fov: f32 = std::f32::consts::PI / 2.0;

	let mut perspective = display_math::perspective_matrix(1, 1, fov);

	let mut camera = display_math::Camera {
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

		let view = display_math::view_matrix(
			&[camera.loc_x, camera.loc_y, camera.loc_z],
			&[camera.dir_x, camera.dir_y, camera.dir_z],
			&[0.0, 1.0, 0.0]);

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
					perspective = display_math::perspective_matrix(w, h, fov),
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
					display_math::handle_mouse_move(
						&display.get_window().unwrap(), &mut camera, x, y),
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
			let duration = last_time.to(current_time).num_milliseconds() as f32 / 1000.0;
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

struct MovementState {
	forward: bool,
	backward: bool,
	left: bool,
	right: bool
}

