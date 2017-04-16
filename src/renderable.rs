//! Trait to allow objects to render themselves

use glium::DrawParameters;
use glium::Frame;
use glium::Program;
use glium::Surface;
use glium::uniforms::SamplerWrapFunction;
use linear_algebra::{Mat3, Mat4, Vec3};
use model::gpu::ModelInstance;

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

