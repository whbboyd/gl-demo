use errors::*;
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use glium::index::PrimitiveType::TrianglesList;
use glium::texture::{MipmapsOption, Texture2d};
use model::{mem, Vertex};

#[derive(Debug)]
pub struct Geometry {
	pub vertices: VertexBuffer<Vertex>,
	pub indices: IndexBuffer<u16>,
}
impl Geometry {
	pub fn from_mem(display: &Facade, geometry: &mem::Geometry) -> Result<Geometry> {
		Ok( Geometry {
			vertices: try!{ VertexBuffer::new(display, geometry.vertices.as_ref())
					.chain_err(|| "Could not upload vertices to GPU") },
			indices: try!{ IndexBuffer::new(display, TrianglesList, geometry.indices.as_ref())
					.chain_err(|| "Could not upload indices to GPU") },
		} )
	}
}

#[derive(Debug)]
pub struct Material {
	pub ambient: (f32, f32, f32),
	pub specular: (f32, f32, f32),
	pub texture: Texture2d,
}
impl Material {
	pub fn from_mem(display: &Facade, material: &mem::Material) -> Result<Material> {
		let src = material.clone();
		Ok( Material {
			ambient: src.ambient,
			specular: src.specular,
			texture: try!{
				Texture2d::with_mipmaps(display, src.texture, MipmapsOption::NoMipmap)
					.chain_err(|| "Could not upload texture to GPU") },
		} )
	}
}

#[derive(Debug)]
pub struct Model {
	pub geometry: Geometry,
	pub material: Material,
}
impl Model {
	pub fn from_mem(display: &Facade, model: &mem::Model) -> Result<Model> {
		Ok ( Model {
			geometry: try!{ Geometry::from_mem(display, model.geometry.as_ref()) },
			material: try!{ Material::from_mem(display, model.material.as_ref()) },
		} )
	}
}

#[derive(Debug)]
pub struct ModelInstance<'a> {
	pub model: &'a Model,
	pub model_matrix: [[f32; 4]; 4],
}

