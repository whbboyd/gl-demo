#[macro_use]
extern crate glium;
extern crate time;

// This is the hardcoded Utah Teapot model from
// https://tomaka.github.io/glium/book/tuto-07-teapot.rs
#[path = "tuto-07-teapot.rs"]
mod teapot;

mod display_math;
mod shader_source;

use glium::{Depth, DisplayBuild, DrawParameters, Program, Surface};
use glium::{VertexBuffer, IndexBuffer};
use glium::draw_parameters::{DepthTest,BackfaceCullingMode};
use glium::glutin::{ElementState, Event, WindowBuilder};
use glium::index::PrimitiveType::TrianglesList;
use time::PreciseTime;

fn main() {
	println!("Starting demo...");

	println!("Initializing display...");
	let display = WindowBuilder::new()
		.with_depth_buffer(24)
		.with_vsync()
		.build_glium().unwrap();

	println!("Compiling shaders...");
	let program = Program::from_source(
		&display,
		shader_source::VERTEX_SHADER_SRC,
		shader_source::FRAGMENT_SHADER_SRC,
		None).unwrap();

	println!("Preparing environment...");
	let params = DrawParameters {
		depth: Depth {
			test: DepthTest::IfLess,
			write: true,
			.. Default::default()
		},
		backface_culling: BackfaceCullingMode::CullClockwise,
		.. Default::default()
	};

	println!("Building world...");
	let mut objects = Vec::new();
	for x in 0..3 { for y in 0..3 { for z in 0..3 {
		let obx = x as f32 * 1.5;
		let oby = y as f32 * 1.5;
		let obz = z as f32 * 1.5;
		let scale = 0.005 + (obx + oby + obz) / 1500.0;
		objects.push(Object {
			vertices: VertexBuffer::new(&display, &teapot::VERTICES).unwrap(),
			normals: VertexBuffer::new(&display, &teapot::NORMALS).unwrap(),
			indices: IndexBuffer::new(&display, TrianglesList, &teapot::INDICES).unwrap(),
			model_matrix: [
				[scale,	0.0,	0.0,	0.0],
				[0.0,	scale,	0.0,	0.0],
				[0.0,	0.0,	scale,	0.0],
				[obx,	oby,	obz,	1.0] ] } );
	} } };

	let light = [-1.0, 0.4, 0.9f32];

	let mut frame: u64 = 0;
	let mut last_time = PreciseTime::now();

	let fps_message_interval = 500;
	let fov: f32 = std::f32::consts::PI / 2.0;

	let mut perspective = display_math::perspective_matrix(1, 1, fov);

	let mut camera = display_math::Camera {
		loc_x: -5.0,
		loc_y: 0.0,
		loc_z: 0.0,
		dir_x: 1.0,
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

		let view = display_math::view_matrix(
			&[camera.loc_x, camera.loc_y, camera.loc_z],
			&[camera.dir_x, camera.dir_y, camera.dir_z],
			&[0.0, 1.0, 0.0]);

		for object in objects.iter() {
			target.draw(
				(&object.vertices, &object.normals),
				&object.indices,
				&program,
				&uniform! {
					model_matrix: object.model_matrix,
					view_matrix: view,
					perspective_matrix: perspective,
					u_light: light},
				&params).unwrap();
		}

		target.finish().unwrap();

		// Handle events
		for ev in display.poll_events() {
			match ev {
				// Esc or Q:
				Event::KeyboardInput(ElementState::Released, 9, _) |
				Event::KeyboardInput(ElementState::Released, 24, _) |
				Event::Closed =>
					break 'main,
				// Up:
				Event::KeyboardInput(ElementState::Pressed, 25, _) =>
					movement.forward = true,
				Event::KeyboardInput(ElementState::Released, 25, _) =>
					movement.forward = false,
				// Left:
				Event::KeyboardInput(ElementState::Pressed, 38, _) =>
					movement.left = true,
				Event::KeyboardInput(ElementState::Released, 38, _) =>
					movement.left = false,
				// Down:
				Event::KeyboardInput(ElementState::Pressed, 39, _) =>
					movement.backward = true,
				Event::KeyboardInput(ElementState::Released, 39, _) =>
					movement.backward = false,
				// Right:
				Event::KeyboardInput(ElementState::Pressed, 40, _) =>
					movement.right = true,
				Event::KeyboardInput(ElementState::Released, 40, _) =>
					movement.right = false,
				// Space:
				Event::KeyboardInput(ElementState::Released, 65, _) => {
					camera.loc_x = -5.0;
					camera.loc_y = 0.0;
					camera.loc_z = 0.0;
					camera.dir_x = 1.0;
					camera.dir_y = 0.0;
					camera.dir_z = 0.0;
				}
				Event::MouseMoved(x, y) =>
					display_math::handle_mouse_move(
						&display.get_window().unwrap(), &mut camera, x, y),
				Event::Resized(w, h) =>
					perspective = display_math::perspective_matrix(w, h, fov),
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

struct Object {
	vertices: VertexBuffer<teapot::Vertex>,
	normals: VertexBuffer<teapot::Normal>,
	indices: IndexBuffer<u16>,
	model_matrix: [[f32; 4]; 4]
}

