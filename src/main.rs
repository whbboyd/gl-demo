#![warn(missing_docs)]

//! A simple demo OpenGL renderer.
//!
//! This uses the Glium and Glutin libraries to render a simple 3D world,
//! with sketched-out movement and physics.
//!
//! The program expects the following files to be available relative to the
//! working directory:
//!
//!  * `data/fragment_shader.frag`
//!  * `data/materials.mtl`
//!  * `data/wt_teapot.obj`
//!  * `data/floor-texture.png`
//!  * `data/heightmap.png`
//!  * `data/teapot-texture.png`
//!  * `data/vertex_shader.vert`
//!
//! These files are all in these locations relative to the repository root, so
//! running the program from the repository root (e.g. with `cargo run`)
//! will find them where it expects.
//!
//! Movement controls are as follows:
//!
//!  * Mouse: rotate camera
//!  * `W`: move forwards
//!  * `A`: move left
//!  * `S`: move backwards
//!  * `D`: move right
//!  * Space: jump
//!  * `Q`/Esc: exit

#[macro_use]
extern crate error_chain;
extern crate env_logger;
#[macro_use]
extern crate glium;
extern crate image;
#[macro_use]
extern crate log;
extern crate time;
extern crate wavefront_obj;

pub mod display_math;
pub mod linear_algebra;
pub mod model;
pub mod physics;
pub mod renderable;

mod errors { error_chain! { } }

use env_logger::LogBuilder;
use errors::*;
use glium::{Depth, DisplayBuild, DrawParameters, Program, Surface};
use glium::draw_parameters::{BackfaceCullingMode, DepthTest, PolygonMode};
use glium::glutin::{Api, ElementState, Event, GlRequest, WindowBuilder};
use glium::texture::Texture2d;
use linear_algebra::{Mat4, Vec3};
use log::{LogLevel, LogRecord};
use model::heightmap::Heightmap;
use renderable::{Renderable, TextRenderable2d};
use std::fs::File;
use std::io::{BufReader, Read};
use time::{now, PreciseTime};

const TEAPOT_PATH: &'static str = "data/wt-teapot.obj";
const FLOOR_HEIGHTMAP: &'static str = "data/heightmap.png";
const FLOOR_MATERIALS: &'static str = "data/materials.mtl";
const FONT_TEXTURE: &'static str = "data/font-texture.png";
const VERTEX_SHADER_PATH: &'static str = "data/vertex-shader.vert";
const FRAGMENT_SHADER_PATH: &'static str = "data/fragment-shader.frag";

//const CHAR_MAX_SPEED: f32 = 0.2;
const CHAR_MAX_SPEED: f32 = 1.0;
const CHAR_DECEL: f32 = 0.05;
//const CHAR_MAX_JUMP: f32 = 0.2;
const CHAR_MAX_JUMP: f32 = 1.0;
const CHAR_GRAVITY: f32 = 0.02;

/// Main entry point and error handling.
fn main() {
	init_log();
	if let Err(e) = run() {
		error!("Fatal error: {}", e);
		for e in e.iter().skip(1) {
			error!("\tCaused by: {}", e);
		}
		if let Some(backtrace) = e.backtrace() {
			error!("Backtrace: {:?}", backtrace);
		}
		::std::process::exit(1);
	}
}

/// Run function.
///
/// This loads all neccessary world state, then runs the main event loop,
/// which reads input, updates world state, and renders to the window.
fn run() -> Result<()> {
	info!("Starting demo...");

	info!("Initializing display...");
	let display = try!{ WindowBuilder::new()
			.with_depth_buffer(24)
			.with_vsync()
			//TODO What's the minimum version we can get away with?
			//FIXME This isn't behaving as expected.
			.with_gl(GlRequest::Specific(Api::OpenGl, (2, 1)))
			.build_glium()
			.map_err(|e| { Error::from(format!("{:?}", e)) } ) };

	info!("Loading models and textures...");
	let library = model::library::ModelLibrary::new();
	let mut file = try!{ File::open(TEAPOT_PATH).chain_err(|| "Could not load teapot model") };
	let teapot = try!{ library.load_model(&mut file) };
	let mut file = try!{ File::open(FLOOR_MATERIALS)
			.chain_err(|| "Could not load floor materials") };
	let floor_mat = try!{ try!{ model::disk::load_mats(&mut file) }.remove("Floor")
			.ok_or(Error::from("Floor material library missing floor material (\"Floor\")")) };
	let file = try!{ File::open(FLOOR_HEIGHTMAP).chain_err(|| "Could not load heightmap") };
	let heightmap = try!{ model::disk::load_texture(&mut BufReader::new(file))
			.chain_err(|| "Could not load heightmap") };
	let mut floor = model::heightmap::simpleheightmap::SimpleHeightmap::from_map(
			&heightmap,
			0.0,
			100.0,
			-100.0,
			-86.6,
			1.0,
			&display,
			floor_mat);
	let file = try!{ File::open(FONT_TEXTURE).chain_err(|| "Could not load font texture") };
	let font = try!{ model::disk::load_texture(&mut BufReader::new(file))
			.chain_err(|| "Could not load font texture") };
	let font = try!{ Texture2d::new(&display, font)
			.chain_err(|| "Could not load font texture") };

	info!("Loading shaders...");
	let mut vertex_shader = String::new();
	let mut file = try!{ File::open(VERTEX_SHADER_PATH)
			.chain_err(|| "Could not load vertex shader") };
	try!{ file.read_to_string(&mut vertex_shader)
			.chain_err(|| "Could not load vertex shader") };
	let mut fragment_shader = String::new();
	let mut file = try!{ File::open(FRAGMENT_SHADER_PATH)
			.chain_err(|| "Could not load fragment shader") };
	try!{ file.read_to_string(&mut fragment_shader)
			.chain_err(|| "Could not load fragment shader") };

	info!("Compiling shaders...");
	let program = try!{
		Program::from_source(&display, &vertex_shader, &fragment_shader, None)
			.chain_err(|| "Error compiling shaders")
	};

	info!("Preparing environment...");
	let params = DrawParameters {
		depth: Depth {
			test: DepthTest::IfLess,
			write: true,
			.. Default::default()
		},
		backface_culling: BackfaceCullingMode::CullCounterClockwise,
//		polygon_mode: PolygonMode::Line,
		.. Default::default()
	};

	info!("Building world...");
	let gpu_teapot = try!{ model::gpu::Model::from_mem(&display, &teapot) };
	let mut objects = Vec::new();
	for x in 0u8..3 { for y in 0u8..3 { for z in 0u8..3 {
		let obx = x as f32 * 1.5;
		let oby = y as f32 * 1.5;
		let obz = z as f32 * 1.5;
		let scale = 0.5 + (obx + oby + obz) / 30.0;
		objects.push(model::gpu::ModelInstance {
				model: &gpu_teapot,
				model_matrix: Mat4::from( [
					[scale,	0.0,	0.0,	0.0],
					[0.0,	scale,	0.0,	0.0],
					[0.0,	0.0,	scale,	0.0],
					[obx,	oby,	obz,	1.0] ] ), } );
	} } };

	let light_pos = Vec3::from([-1.0, 0.4, 0.9f32]);
	let light_color = (1.0, 1.0, 1.0f32);

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
		Vec3::from([-5.0, 0.0, 0.0]),
		Vec3::from([0.0, 0.0, 0.0]),
		CHAR_MAX_SPEED,
		CHAR_DECEL,
		CHAR_MAX_JUMP,
		CHAR_GRAVITY);

	let mut camera = display_math::Camera {
		loc: character.loc().clone(),
		dir: Vec3::from([1.0, 0.0, 0.0]),
	};
	camera.loc[1] += 0.5;
	floor.update_lod(&camera.loc);

	// Main program loop
	info!("Starting program loop...");
	'main: loop {
		frame += 1;

		let mut target = display.draw();
		target.clear_color_and_depth((0.5, 0.5, 1.0, 1.0), 1.0);

		let view = display_math::view_matrix(
			camera.loc,
			camera.dir,
			Vec3::from([0.0, 1.0, 0.0]));

		let render_state = renderable::DefaultRenderState {
			view: view,
			perspective: perspective,
			light_pos: light_pos,
			light_color: light_color,
			params: &params,
			program: &program,
		};

		for object in objects.iter() {
			object.render(&render_state, &mut target);
		}
		floor.render(&render_state, &mut target);

		//TODO
		let current_time = PreciseTime::now();
		let duration = last_time.to(current_time).num_milliseconds() as f32 / 1000.0;
		let frames = frame % fps_message_interval;
		let fps = frames as f32 / duration;
		let hud_text = format!("fps: {:.1}, loc: {:.1},{:.1},{:.1}, dir: {:.1},{:.1},{:.1}",
				fps,
				character.loc()[0], character.loc()[1], character.loc()[2],
				camera.dir[0], camera.dir[1], camera.dir[2])
				.to_string().into_bytes();
		let hud = TextRenderable2d::new(hud_text, &font, 16);
		hud.render(&render_state, &mut target);

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
				Event::MouseMoved(x, y) =>
					try!{ display_math::handle_mouse_move(
							&display.get_window().unwrap(), &mut camera, x, y)
					},
				Event::Resized(w, h) =>
					perspective = display_math::perspective_matrix(w, h, fov),
				_ => ()
			}
		}

		character.do_char_movement(&camera.dir, &mut movement, &floor);

		// Update camera
		camera.loc = character.loc().clone();
		camera.loc[1] += 0.5;
		floor.update_lod(&camera.loc);

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

	Ok(())
}

/// Struct to hold character movement state.
#[derive(Debug)]
pub struct MovementState {
	/// True if this character is attempting to move forwards.
	pub forward: bool,
	/// True if this character is attempting to move backwards.
	pub backward: bool,
	/// True if this character is attempting to strafe left.
	pub left: bool,
	/// True if this character is attempting to strafe right.
	pub right: bool,
	/// True if this character is attempting to jump.
	pub jumping: bool,
	/// Number of frames this character can continue to accelerate while
	/// jumping.
	pub can_jump: u8
}

/// Configure logging.
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
