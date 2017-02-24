//! Objects that have been uploaded to GPU memory for rendering.

use errors::*;
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use glium::index::PrimitiveType::TrianglesList;
use glium::texture::Texture2d;
use linear_algebra::Mat4;
use model::{mem, Vertex};

/// GPU geometry, that is `Vertex`s.
#[derive(Debug)]
pub struct Geometry {
	/// The uploaded vertex buffer.
	pub vertices: VertexBuffer<Vertex>,
	/// The uploaded index buffer.
	pub indices: IndexBuffer<u32>,
}
impl Geometry {
	/// Upload an in-memory `model::mem::Geometry` to GPU memory.
	pub fn from_mem(display: &Facade, geometry: &mem::Geometry) -> Result<Geometry> {
		Ok( Geometry {
			vertices: try!{ VertexBuffer::new(display, geometry.vertices.as_ref())
					.chain_err(|| "Could not upload vertices to GPU") },
			indices: try!{ IndexBuffer::new(display, TrianglesList, geometry.indices.as_ref())
					.chain_err(|| "Could not upload indices to GPU") },
		} )
	}
}

/// GPU materials.
#[derive(Debug)]
pub struct Material {
	/// The object's ambient color.
	pub ambient: (f32, f32, f32),
	/// The object's specular color.
	pub specular: (f32, f32, f32),
	/// The uploaded texture buffer.
	pub texture: Texture2d,
}
impl Material {
	/// Upload the texture from an in-memory `model::mem::Material` to GPU
	/// memory.
	pub fn from_mem(display: &Facade, material: &mem::Material) -> Result<Material> {
		let src = material.clone();
		Ok( Material {
			ambient: src.ambient,
			specular: src.specular,
			texture: try!{
				Texture2d::new(display, src.texture)
					.chain_err(|| "Could not upload texture to GPU") },
		} )
	}
}

/// A full model, including geometry and material.
#[derive(Debug)]
pub struct Model {
	/// The uploaded geometry.
	pub geometry: Geometry,
	/// The uploaded material.
	pub material: Material,
}
impl Model {
	/// Upload geometry and textures from an in-memory `model::mem::Model` to
	/// GPU memory.
	pub fn from_mem(display: &Facade, model: &mem::Model) -> Result<Model> {
		Ok ( Model {
			geometry: try!{ Geometry::from_mem(display, model.geometry.as_ref()) },
			material: try!{ Material::from_mem(display, model.material.as_ref()) },
		} )
	}
}

/// An in-world instance of an uploaded model.
#[derive(Debug)]
pub struct ModelInstance<'a> {
	/// The model in question.
	pub model: &'a Model,
	/// The transformation matrix to place the model in the world.
	pub model_matrix: Mat4<f32>,
}

