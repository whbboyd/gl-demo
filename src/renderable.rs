//! Trait to allow objects to render themselves

use glium::{BlitTarget, DrawParameters, Frame, Program, Rect, Surface};
use glium::backend::Facade;
use glium::texture::Texture2d;
use glium::uniforms::{MagnifySamplerFilter, SamplerWrapFunction};
use linear_algebra::{Mat3, Mat4, Vec3};
use model::Vertex;
use model::gpu;
use model::gpu::ModelInstance;
use model::mem;

/// Trait for an object which may be rendered.
///
/// Takes parameterized state (typically stuff like view matrix) and a drawing
/// target and renders itself to the target using the state.
pub trait Renderable<Param, Target> {
	/// Render this object to the given target using the given state.
	fn render(&self, renderState: Param, target: Target);
}

/// Struct to hold render state for a typical OpenGL 3D object.
pub struct DefaultRenderState<'a> {
	/// View matrix
	pub view: Mat4<f32>,
	/// Perspective matrix
	pub perspective: Mat4<f32>,
	/// Direction of the global light
	pub light_pos: Vec3<f32>,
	/// Color of the global light
	pub light_color: (f32, f32, f32),
	/// OpenGL drawing parameters
	pub params: &'a DrawParameters<'a>,
	/// Shader program to run
	pub program: &'a Program,
}

/// Default implementation for model::gpu::ModelInstances.
impl<'a> Renderable<&'a DefaultRenderState<'a>, &'a mut Frame> for ModelInstance<'a> {

	/// Render this ModelInstance.
	///
	/// This computes model/view, model/view/perspective, normal and lighting
	/// matrices and uses them to 3D render the model instance to the target.
	fn render(&self, renderstate: &DefaultRenderState, target: &mut Frame) {
		let light_vector_raw: [f32; 3] = renderstate.light_pos.into();
		let x: Mat3<f32> = renderstate.view.into();
		let light_matrix_raw: [[f32; 3]; 3] = x.into();
		let model_view = self.model_matrix * renderstate.view;
		let model_view_perspective_raw: [[f32; 4]; 4] =
				(model_view * renderstate.perspective).into();
		let x: Mat3<f32> = model_view.into();
		let normal_raw: [[f32; 3]; 3] = x.into();
		target.draw(
			&self.model.geometry.vertices,
			&self.model.geometry.indices,
			renderstate.program,
			&uniform! {
				model_view_perspective_matrix: model_view_perspective_raw,
				normal_matrix: normal_raw,
				light_matrix: light_matrix_raw,
				u_light_pos: light_vector_raw,
				u_light_color: renderstate.light_color,
				u_mat_ambient: self.model.material.ambient,
				u_mat_specular: self.model.material.specular,
				u_mat_texture: self.model.material.texture
					.sampled().wrap_function(SamplerWrapFunction::Repeat),
				},
			renderstate.params).unwrap();
	}
}


/// Render text to the screen
pub struct TextRenderable2d<'a> {
	text: Vec<u8>,
	font: &'a Texture2d,
	chars_wide: u8,
	chars_high: u8,
	char_width: u32,
	char_height: u32,
}

impl<'a> TextRenderable2d<'a> {
	/// Create a new TextRenderable2d containing the given text in the given
	/// font (which is the given number of characters wide).
	pub fn new(text: Vec<u8>, font: &Texture2d, chars_wide: u8) -> TextRenderable2d {
		let chars_high = (256 / chars_wide as u16) as u8;
		let char_width = font.width() / chars_wide as u32;
		let char_height = font.height() / chars_high as u32;
		TextRenderable2d {
			text: text,
			font: font,
			chars_wide: chars_wide,
			chars_high: chars_high,
			char_width: char_width,
			char_height: char_height,
		}
	}
}

impl<'a> Renderable<&'a DefaultRenderState<'a>, &'a mut Frame> for TextRenderable2d<'a> {
	fn render(&self, _: &DefaultRenderState, target: &mut Frame) {
		let font_surface = &self.font.as_surface();
		let mut idx = 0u32;
		for character in self.text.iter() {
			let char_origin_x = (character % self.chars_wide) as u32 * self.char_width;
			let char_origin_y = (self.chars_high - character / self.chars_high - 1) as u32 *
					self.char_height;
			target.blit_from_simple_framebuffer(
					font_surface,
					&Rect {left: char_origin_x,
							bottom: char_origin_y,
							width: self.char_width,
							height: self.char_height },
					&BlitTarget {left: idx * self.char_width,
							bottom: target.get_dimensions().1 - self.char_height,
							width: self.char_width as i32,
							height: self.char_height as i32 },
					MagnifySamplerFilter::Linear);

			idx += 1;
		}
	}
}

/// Create a 3D model flag containing the given text in the given font.
pub fn text_model(
		text: Vec<u8>,
		font: &Texture2d,
		display: &Facade,
		chars_wide: u8,
		loc: Vec3<f32>)
		-> gpu::Model {
	let chars_high = (256 / chars_wide as u16) as u8;
	let char_width = font.width() / chars_wide as u32;
	let char_height = font.height() / chars_high as u32;
	let font_surface = font.as_surface();
	let text_texture = Texture2d::empty(
			display,
			text.len() as u32 * char_width,
			char_height).unwrap();
	// Draw text onto text_texture
	{
		let mut text_surface = text_texture.as_surface();
		text_surface.clear_color(0.0, 0.0, 0.0, 1.0);
		let mut idx = 0u32;
		for character in text.iter() {
			let char_origin_x = (character % chars_wide) as u32 * char_width;
			let char_origin_y = (chars_high - character / chars_high - 1) as u32 *
					char_height;
			text_surface.blit_from_simple_framebuffer(
					&font_surface,
					&Rect {left: char_origin_x,
							bottom: char_origin_y,
							width: char_width,
							height: char_height },
					&BlitTarget {left: idx * char_width,
							bottom: 0,
							width: char_width as i32,
							height: char_height as i32 },
					MagnifySamplerFilter::Linear);
			idx += 1;
		}
	}
//XXX Okay, we've just drawn. Something doesn't sync if we don't fiddle with the text.
text_texture.read_to_pixel_buffer();
	// Build model
	let w = text.len() as f32 * 5.0;
	let geom = mem::Geometry {
		vertices: vec![
		Vertex {
				position: [loc[0], loc[1], loc[2]],
				normal: [1.0, 0.0, 0.0],
				tex_uv: [0.0, 0.0] },
		Vertex {
				position: [loc[0], loc[1] + 5.0, loc[2]],
				normal: [1.0, 0.0, 0.0],
				tex_uv: [0.0, 1.0] },
		Vertex {
				position: [loc[0], loc[1] + 5.0, loc[2] + w],
				normal: [1.0, 0.0, 0.0],
				tex_uv: [1.0, 1.0] },
		Vertex {
				position: [loc[0], loc[1], loc[2] + w],
				normal: [1.0, 0.0, 0.0],
				tex_uv: [1.0, 0.0] },
		],
		// TODO: The back is backwards. Good enough for now.
		indices: vec![0, 1, 2, 2, 3, 0, 2, 1, 0, 0, 3, 2],
	};
	// Upload text object to GPU and return
	let gpu_geom = gpu::Geometry::from_mem(display, &geom).unwrap();
	let gpu_mat = gpu::Material {
		ambient: (1.0, 1.0, 1.0),
		specular: (0.0, 0.0, 0.0),
		texture: text_texture,
	};
	gpu::Model { geometry: gpu_geom, material: gpu_mat }
}
