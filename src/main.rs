#[macro_use]
extern crate glium;
extern crate image;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate time;
extern crate wavefront_obj;

mod display_math;
mod model;
mod physics;

use env_logger::LogBuilder;
use glium::{Depth, DisplayBuild, DrawParameters, Program, Surface};
use glium::draw_parameters::{DepthTest,BackfaceCullingMode};
use glium::glutin::{Api, ElementState, Event, GlRequest, WindowBuilder};
use log::{LogLevel, LogRecord};
use std::fs::File;
use std::io::Read;
use time::{now, PreciseTime};

const TEAPOT_PATH: &'static str = "data/wt_teapot.obj";
const FLOOR_PATH: &'static str = "data/floor.obj";
const VERTEX_SHADER_PATH: &'static str = "data/vertex_shader.vert";
const FRAGMENT_SHADER_PATH: &'static str = "data/fragment_shader.frag";

const CHAR_MAX_SPEED: f32 = 0.2;
const CHAR_DECEL: f32 = 0.05;
const CHAR_MAX_JUMP: f32 = 0.2;
const CHAR_GRAVITY: f32 = 0.02;

fn main() {
	init_log();
	info!("Starting demo...");

	info!("Loading models and textures...");
	let library = model::mem::ModelLibrary::new();
	let mut file = File::open(TEAPOT_PATH).unwrap();
	let teapot = library.load_model(&mut file);
	let mut file = File::open(FLOOR_PATH).unwrap();
	let floor = library.load_model(&mut file);

	info!("Initializing display...");
	let display = WindowBuilder::new()
		.with_depth_buffer(24)
		.with_vsync()
		//TODO What's the minimum version we can get away with?
		//FIXME This isn't behaving as expected.
		.with_gl(GlRequest::Specific(Api::OpenGl, (2, 1)))
		.build_glium().unwrap();

	info!("Loading shaders...");
	let mut file = File::open(VERTEX_SHADER_PATH).unwrap();
	let mut vertex_shader = String::new();
	file.read_to_string(&mut vertex_shader).unwrap();
	let mut file = File::open(FRAGMENT_SHADER_PATH).unwrap();
	let mut fragment_shader = String::new();
	file.read_to_string(&mut fragment_shader).unwrap();

	info!("Compiling shaders...");
	let program = Program::from_source(
		&display, &vertex_shader, &fragment_shader, None).unwrap();

	info!("Preparing environment...");
	let params = DrawParameters {
		depth: Depth {
			test: DepthTest::IfLess,
			write: true,
			.. Default::default()
		},
		backface_culling: BackfaceCullingMode::CullClockwise,
		.. Default::default()
	};

	info!("Building world...");
	let gpu_teapot = model::gpu::Model::from_mem(&display, &teapot);
	let gpu_floor = model::gpu::Model::from_mem(&display, &floor);
	let mut objects = Vec::new();
	for x in 0u8..3 { for y in 0u8..3 { for z in 0u8..3 {
		let obx = x as f32 * 1.5;
		let oby = y as f32 * 1.5;
		let obz = z as f32 * 1.5;
		let scale = 0.5 + (obx + oby + obz) / 30.0;
		objects.push(model::gpu::ModelInstance {
				model: &gpu_teapot,
				model_matrix: [
					[scale,	0.0,	0.0,	0.0],
					[0.0,	scale,	0.0,	0.0],
					[0.0,	0.0,	scale,	0.0],
					[obx,	oby,	obz,	1.0] ], } );
	} } };
	objects.push(model::gpu::ModelInstance {
		model: &gpu_floor,
		model_matrix: [
			[999.0,	0.0,	0.0,	0.0],
			[0.0,	999.0,	0.0,	0.0],
			[0.0,	0.0,	999.0,	0.0],
			[0.0,	-0.5,	0.0,	1.0] ], } );

	let light = [-1.0, 0.4, 0.9f32];

	let mut frame: u64 = 0;
	let mut last_time = PreciseTime::now();

	let fps_message_interval = 500;
	let fov: f32 = std::f32::consts::PI / 2.0;

	let mut perspective = display_math::perspective_matrix(1, 1, fov);

	let mut movement = MovementState {
		forward: false,
		backward: false,
		left: false,
		right: false,
		jumping: false,
		can_jump: 0
	};

	let mut character = physics::CharacterState::new(
		[-5.0, 0.0, 0.0],
		[0.0, 0.0, 0.0],
		CHAR_MAX_SPEED,
		CHAR_DECEL,
		CHAR_MAX_JUMP,
		CHAR_GRAVITY);

	let mut camera = display_math::Camera {
		loc: character.loc().clone(),
		dir: [1.0, 0.0, 0.0]
	};
	camera.loc[1] += 0.5;

	// Main program loop
	info!("Starting program loop...");
	'main: loop {
		frame += 1;

		let mut target = display.draw();
		target.clear_color_and_depth((0.5, 0.5, 1.0, 1.0), 1.0);

		let view = display_math::view_matrix(
			&camera.loc,
			&camera.dir,
			&[0.0, 1.0, 0.0]);

		for object in objects.iter() {
			target.draw(
				(&object.model.geometry.vertices, &object.model.geometry.normals),
				&object.model.geometry.indices,
				&program,
				&uniform! {
					model_matrix: object.model_matrix,
					view_matrix: view,
					perspective_matrix: perspective,
					u_light: light,
					u_mat_ambient: object.model.material.ambient,
					u_mat_diffuse: object.model.material.diffuse,
					u_mat_specular: object.model.material.specular},
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
				Event::KeyboardInput(ElementState::Pressed, 65, _) =>
					movement.jumping = true,
				Event::KeyboardInput(ElementState::Released, 65, _) => {
					movement.jumping = false;
					movement.can_jump = 0;
				}
				// Enter:
				Event::KeyboardInput(ElementState::Pressed, 36, _) => (),
				Event::KeyboardInput(ElementState::Released, 36, _) =>
					info!("\t{:?}\n\t{:?}\n\t{:?}\n\tSpeed: {}", camera, character, movement,
						character.speed()),
				Event::MouseMoved(x, y) =>
					display_math::handle_mouse_move(
						&display.get_window().unwrap(), &mut camera, x, y),
				Event::Resized(w, h) =>
					perspective = display_math::perspective_matrix(w, h, fov),
				_ => ()
			}
		}

		physics::do_char_movement(&mut character, &camera.dir, &mut movement);

		// Update camera
		camera.loc = character.loc().clone();
		camera.loc[1] += 0.5;

		// Wait for end of frame
		// We enabled vsync when creating the window, so this happens automatically.

		if frame % fps_message_interval == 0 {
			let current_time = PreciseTime::now();
			let duration = last_time.to(current_time).num_milliseconds() as f32 / 1000.0;
			let fps = fps_message_interval as f32 / duration;
			last_time = current_time;
			info!("Rendered {} frames in {} seconds ({} FPS)",
				fps_message_interval,
				duration,
				fps);
		}
	}

	info!("Program loop ended, exiting...");
}

#[derive(Debug)]
pub struct MovementState {
	pub forward: bool,
	pub backward: bool,
	pub left: bool,
	pub right: bool,
	pub jumping: bool,
	pub can_jump: u8
}

fn init_log() {
	let mut builder = LogBuilder::new();
	builder.filter(None, LogLevel::Info.to_log_level_filter());
	builder.format(|record: &LogRecord| {
		format!("[{}] [{} {}:{}] [{}] {}",
			now().rfc3339(),
			record.location().module_path(),
			record.location().file(),
			record.location().line(),
			record.level(),
			record.args()) } );
	builder.init().unwrap();
}
