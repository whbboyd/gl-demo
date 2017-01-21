use glium::texture::RawImage2d;
use image;
use model::mem;
use model::{Normal, Vertex};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use wavefront_obj::{ParseError, obj, mtl};

#[derive(Debug)]
pub enum LoadModelError {
	IOError(io::Error),
	ParseError(ParseError)
}

pub fn load_model(read: &mut io::Read) ->
		Result<(mem::Geometry, mem::Material), LoadModelError> {
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
			.map(|fname| load_mats(&mut File::open(fname).unwrap()).unwrap() )
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
	let mut mat: mem::Material = mem::DEFAULT_MAT;
	for geom in object.geometry {
		mat = match geom.material_name {
			Some(m) => mats.get(&m).unwrap_or_else(|| {
				error!("Missing material: {:?}", &m);
				&mat }).clone(),
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

	Ok( (mem::Geometry { vertices: vertices,
			normals: normals,
			indices: indices, },
		mat) )
}

pub fn load_mats(read: &mut io::Read) -> Result<HashMap<String, mem::Material>, LoadModelError> {
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
		let converted_mat = mem::Material::from(&mat);
		mats.insert(mat.name, converted_mat);
	}
	Ok(mats)
}

impl <'a> From<&'a mtl::Material> for mem::Material {
	fn from(mat: &mtl::Material) -> Self {
		mem::Material {
			ambient: color_conv(mat.color_ambient),
			diffuse: color_conv(mat.color_diffuse),
			specular: color_conv(mat.color_specular),
			texture: None
		}
	}
}
fn color_conv(color: mtl::Color) -> (f32, f32, f32) {
	(color.r as f32, color.g as f32, color.b as f32)
}

#[derive(Debug)]
pub enum LoadTextureError {
	IOError(io::Error)
}

pub fn load_texture<T>(read: &mut T)
		-> Result<RawImage2d<u8>, LoadTextureError>
		where T: io::BufRead + io::Seek {
	let image = image::load(read, image::PNG).unwrap().to_rgba();
	let image_dimensions = image.dimensions();
	Ok(RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions))
}

