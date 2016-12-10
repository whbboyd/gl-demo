use geometry::{Material, Normal, Vertex};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Read;
use wavefront_obj::{ParseError, obj, mtl};

const DEFAULT_MAT: Material = Material {
	ambient: (0.0, 0.0, 0.0),
	diffuse: (1.0, 0.0, 1.0),
	specular: (0.0, 1.0, 0.0),
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

	let mats = loaded_object.material_library
			.map(|fname| { load_mats(&mut File::open(fname).unwrap()).unwrap() } )
			.unwrap_or(HashMap::new());

	let object = loaded_object.objects.pop().unwrap();
	let vertices = object.vertices.iter()
		.map(|v| Vertex{position: (v.x as f32, v.y as f32, v.z as f32)})
		.collect::<Vec<Vertex>>();
	let input_normals = object.normals.iter()
		.map(|v| Normal{normal: (v.x as f32, v.y as f32, v.z as f32)})
		.collect::<Vec<Normal>>();
	let mut indices: Vec<u16> = Vec::new();
	let mut normals: Vec<Normal> = Vec::new();
	let mut mat: Material = DEFAULT_MAT;
	for geom in object.geometry {
		mat = match geom.material_name {
			Some(m) => mats.get(&m).unwrap_or_else(|| {
				error!("Missing material: {:?}", &m);
				&mat
			}).clone(),
			None => mat
		};
		for shape in geom.shapes {
			match shape.primitive {
				obj::Primitive::Triangle(a, b, c) => {
					indices.push(c.0 as u16);
					//FIXME: This is a terrible hack and there must be a better way.
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
			material: mat } )

}

pub fn load_mats(read: &mut Read) -> Result<HashMap<String, Material>, LoadModelError> {
	let mut mat_str = String::new();
	try!{
		read.read_to_string(&mut mat_str)
		.or_else(|e| Err(LoadModelError::IOError(e)))
	};
	let loaded_mats = try!{
		mtl::parse(mat_str.into())
		.or_else(|e| Err(LoadModelError::ParseError(e)))
	};
	let mut mats = HashMap::with_capacity(loaded_mats.materials.len());
	for mat in loaded_mats.materials {
		let converted_mat = Material::from(&mat);
		mats.insert(mat.name, converted_mat);
	}
	Ok(mats)
}

impl<'a> From<&'a mtl::Material> for Material {
	fn from(mat: &mtl::Material) -> Self {
		Material {
			ambient: color_conv(mat.color_ambient),
			diffuse: color_conv(mat.color_diffuse),
			specular: color_conv(mat.color_specular)
		}
	}
}
fn color_conv(color: mtl::Color) -> (f32, f32, f32) {
	(color.r as f32, color.g as f32, color.b as f32)
}

