use geometry::{Material, Normal, Vertex};
use std::io;
use std::io::Read;
use wavefront_obj::ParseError;
use wavefront_obj::obj;

// This is the hardcoded Utah Teapot model from
// https://tomaka.github.io/glium/book/tuto-07-teapot.rs
use teapot;

pub use models::FLOOR_VERTICES as floor_vertices;
pub use models::FLOOR_NORMALS as floor_normals;
pub use models::FLOOR_INDICES as floor_indices;
pub use models::FLOOR_MATERIAL as floor_mat;

pub const FLOOR_VERTICES: [Vertex; 4] = [
    Vertex { position: (1.0, 0.0, 1.0) },
    Vertex { position: (1.0, 0.0, -1.0) },
    Vertex { position: (-1.0, 0.0, -1.0) },
    Vertex { position: (-1.0, 0.0, 1.0) }
];

pub const FLOOR_NORMALS: [Normal; 4] = [
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) }
];

pub const FLOOR_INDICES: [u16; 6] = [
	2, 1, 0,
	3, 2, 0
];

pub const FLOOR_MATERIAL: Material = Material {
	light: (0.75, 0.75, 0.75),
	dark: (0.25, 0.25, 0.25),
};

#[derive(Debug)]
pub struct Model {
	pub vertices: Vec<Vertex>,
	pub normals: Vec<Normal>,
	pub indices: Vec<u16>,
	pub material: Material
}

#[derive(Debug)]
pub enum LoadModelError {
	IOError(io::Error),
	ParseError(ParseError)
}

pub fn load_model(read: &mut Read) -> Result<Model, LoadModelError> {
	let mut object_str = String::new();
	try!{
		read.read_to_string(&mut object_str)
		.or_else(|e| Err(LoadModelError::IOError(e)))
	};
	let mut loaded_object = try!{
		obj::parse(object_str.into())
		.or_else(|e| Err(LoadModelError::ParseError(e)))
	};

	let object = loaded_object.objects.pop().unwrap();
	let vertices = object.vertices.iter()
		.map(|v| Vertex{position: (v.x as f32, v.y as f32, v.z as f32)})
		.collect::<Vec<Vertex>>();
	let input_normals = object.normals.iter()
		.map(|v| Normal{normal: (v.x as f32, v.y as f32, v.z as f32)})
		.collect::<Vec<Normal>>();
	let mut indices: Vec<u16> = Vec::new();
	let mut normals: Vec<Normal> = Vec::new();
	for geom in object.geometry {
		for shape in geom.shapes {
			match shape.primitive {
				obj::Primitive::Triangle(a, b, c) => {
					indices.push(c.0 as u16);
					while normals.len() <= c.0 { normals.push(Normal{normal: (0.0, 0.0, 0.0)})}
					normals[c.0] = input_normals[c.2.unwrap()];
					indices.push(b.0 as u16);
					while normals.len() <= b.0 { normals.push(Normal{normal: (0.0, 0.0, 0.0)})}
					normals[b.0] = input_normals[b.2.unwrap()];
					indices.push(a.0 as u16);
					while normals.len() <= a.0 { normals.push(Normal{normal: (0.0, 0.0, 0.0)})}
					normals[a.0] = input_normals[a.2.unwrap()];
				}
				x => warn!("Unsupported primitive: {:?}", x)
			}
		}
	}

	Ok(Model { vertices: vertices,
			normals: normals,
			indices: indices,
			material: teapot::MATERIAL } )

//	Ok(Model { vertices: Box::new(teapot::VERTICES),
//			normals: Box::new(teapot::NORMALS),
//			indices: Box::new(teapot::INDICES),
//			material: teapot::MATERIAL } )

}
