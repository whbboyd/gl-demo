use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use glium::index::PrimitiveType::TrianglesList;
use glium::texture::Texture2d;
use model::mem;
use model::{Normal, Vertex};

#[derive(Debug)]
pub struct Geometry {
	pub vertices: VertexBuffer<Vertex>,
	pub normals: VertexBuffer<Normal>,
	pub indices: IndexBuffer<u16>,
}
impl Geometry {
	pub fn from_mem(display: &Facade, geometry: &mem::Geometry) -> Geometry {
		Geometry {
			vertices: VertexBuffer::new(display, geometry.vertices.as_ref()).unwrap(),
			normals: VertexBuffer::new(display, geometry.normals.as_ref()).unwrap(),
			indices: IndexBuffer::new(display, TrianglesList, geometry.indices.as_ref()).unwrap(),
		}
	}
}

#[derive(Debug)]
pub struct Material {
	pub ambient: (f32, f32, f32),
	pub diffuse: (f32, f32, f32),
	pub specular: (f32, f32, f32),
	pub texture: Option<Texture2d>,
}
impl Material {
	pub fn from_mem(display: &Facade, material: &mem::Material) -> Material {
		let src = material.clone();
		Material {
			ambient: src.ambient.clone(),
			diffuse: src.diffuse.clone(),
			specular: src.specular.clone(),
			texture: src.texture.and_then(|t| Texture2d::new(display, t).ok()),
		}
	}
}


#[derive(Debug)]
pub struct Model {
	pub geometry: Geometry,
	pub material: Material,
	pub model_matrix: [[f32; 4]; 4],
}
impl Model {
	pub fn from_mem(display: &Facade, model: &mem::Model, model_matrix: [[f32; 4]; 4]) ->
			Model {
		Model {
			geometry: Geometry::from_mem(display, model.geometry.as_ref()),
			material: Material::from_mem(display, model.material.as_ref()),
			model_matrix: model_matrix,
		}
	}
}


