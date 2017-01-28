use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use glium::index::PrimitiveType::TrianglesList;
use glium::texture::Texture2d;
use model::{mem, Vertex};

#[derive(Debug)]
pub struct Geometry {
	pub vertices: VertexBuffer<Vertex>,
	pub indices: IndexBuffer<u16>,
}
impl Geometry {
	pub fn from_mem(display: &Facade, geometry: &mem::Geometry) -> Geometry {
		Geometry {
			vertices: VertexBuffer::new(display, geometry.vertices.as_ref()).unwrap(),
			indices: IndexBuffer::new(display, TrianglesList, geometry.indices.as_ref()).unwrap(),
		}
	}
}

#[derive(Debug)]
pub struct Material {
	pub ambient: (f32, f32, f32),
	pub specular: (f32, f32, f32),
	pub texture: Texture2d,
}
impl Material {
	pub fn from_mem(display: &Facade, material: &mem::Material) -> Material {
		let src = material.clone();
		Material {
			ambient: src.ambient.clone(),
			specular: src.specular.clone(),
			texture: Texture2d::with_mipmaps(display, src.texture.clone(), ::glium::texture::MipmapsOption::NoMipmap).unwrap(),
		}
	}
}

#[derive(Debug)]
pub struct Model {
	pub geometry: Geometry,
	pub material: Material,
}
impl Model {
	pub fn from_mem(display: &Facade, model: &mem::Model) -> Model {
		Model {
			geometry: Geometry::from_mem(display, model.geometry.as_ref()),
			material: Material::from_mem(display, model.material.as_ref()),
		}
	}
}

#[derive(Debug)]
pub struct ModelInstance<'a> {
	pub model: &'a Model,
	pub model_matrix: [[f32; 4]; 4],
}

