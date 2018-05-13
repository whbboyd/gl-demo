//! Trait to allow objects to render themselves

use glium::{BlitTarget, DrawParameters, Frame, Program, Rect, Surface};
use glium::texture::Texture2d;
use glium::uniforms::{MagnifySamplerFilter, SamplerWrapFunction};
use linear_algebra::{Mat3, Mat4, Vec3};
use model::gpu::ModelInstance;

/// Trait for an object which may be rendered.
///
/// Takes parameterized state (typically stuff like view matrix) and a drawing
/// target and renders itself to the target using the state.
pub trait Renderable<Param, Target> {
	/// Render this object to the given target using the given state.
	fn render(&self, render_state: Param, target: Target);
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
	fn render(&self, render_state: &DefaultRenderState, target: &mut Frame) {
		let light_vector_raw: [f32; 3] = render_state.light_pos.into();
		let x: Mat3<f32> = render_state.view.into();
		let light_matrix_raw: [[f32; 3]; 3] = x.into();
		let model_view = self.model_matrix * render_state.view;
		let model_view_perspective_raw: [[f32; 4]; 4] =
				(model_view * render_state.perspective).into();
		let x: Mat3<f32> = model_view.into();
		let normal_raw: [[f32; 3]; 3] = x.into();
		target.draw(
			&self.model.geometry.vertices,
			&self.model.geometry.indices,
			render_state.program,
			&uniform! {
				model_view_perspective_matrix: model_view_perspective_raw,
				normal_matrix: normal_raw,
				light_matrix: light_matrix_raw,
				u_light_pos: light_vector_raw,
				u_light_color: render_state.light_color,
				u_mat_ambient: self.model.material.ambient,
				u_mat_specular: self.model.material.specular,
				u_mat_texture: self.model.material.texture
					.sampled().wrap_function(SamplerWrapFunction::Repeat),
				},
			render_state.params).unwrap();
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
