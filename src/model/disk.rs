use image;
use model::{mem, Vertex};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use wavefront_obj::{ParseError, obj, mtl};

#[derive(Debug)]
pub enum LoadModelError {
	IOError(io::Error),
	ParseError(ParseError),
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
	let mut vertices = object.vertices.iter()
		.map(|v| Vertex{position: (v.x as f32, v.y as f32, v.z as f32),
			normal: (0.0, 1.0, 0.0),
			tex_uv: (0.0, 0.0)})
		.collect::<Vec<Vertex>>();
	let normals = object.normals.iter()
		.map(|n| (n.x as f32, n.y as f32, n.z as f32))
		.collect::<Vec<_>>();
	let tex_uv = object.tex_vertices.iter()
		//TODO: Is a texture w a common or useful thing?
		.map(|t| (t.u as f32, t.v as f32))
		.collect::<Vec<_>>();
	let mut indices: Vec<u16> = Vec::new();
	let mut mat: mem::Material = mem::default_mat();
	for geom in object.geometry {
		//TODO: Figure out the ownership to avoid the unneeded clone
		mat = match geom.material_name {
			Some(m) => mats.get(&m).unwrap_or_else(|| {
				error!("Missing material: {:?}", &m);
				&mat }).clone(),
			None => mat,
		};
		for shape in geom.shapes {
			match shape.primitive {
				obj::Primitive::Triangle(a, b, c) => {
					//FIXME: wavefront obj is excessively flexible about
					// indexing normals and texture UV. If anybody actually
					// uses those capabilities, this will break silently.
					indices.push(a.0 as u16);
					if let Some(i) = a.1 { vertices[a.0].tex_uv = tex_uv[i]; }
					if let Some(i) = a.2 { vertices[a.0].normal = normals[i]; }
					indices.push(b.0 as u16);
					if let Some(i) = b.1 { vertices[b.0].tex_uv = tex_uv[i]; }
					if let Some(i) = b.2 { vertices[b.0].normal = normals[i]; }
					indices.push(c.0 as u16);
					if let Some(i) = c.1 { vertices[c.0].tex_uv = tex_uv[i]; }
					if let Some(i) = c.2 { vertices[c.0].normal = normals[i]; }
				}
				x => warn!("Unsupported primitive: {:?}", x)
			}
		}
	}

	Ok( (mem::Geometry { vertices: vertices, indices: indices, }, mat) )
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
		let texture = mat.uv_map.clone().map(|fname|
			load_texture(&mut io::BufReader::new(File::open(fname).unwrap())).unwrap()).unwrap();
		mem::Material {
			ambient: color_conv(mat.color_ambient),
			specular: color_conv(mat.color_specular),
			texture: texture 
		}
	}
}
fn color_conv(color: mtl::Color) -> (f32, f32, f32) {
	(color.r as f32, color.g as f32, color.b as f32)
}

#[derive(Debug)]
pub enum LoadTextureError {
	IOError(io::Error),
	DecodeError(image::ImageError),
}

pub fn load_texture<T>(read: &mut T)
		-> Result<Vec<Vec<(u8, u8, u8, u8)>>, LoadTextureError>
		where T: io::BufRead + io::Seek {
	let image = try!{
		image::load(read, image::PNG)
		.or_else(|e| match e {
			image::ImageError::IoError(io_err) => Err(LoadTextureError::IOError(io_err)),
			other_err => Err(LoadTextureError::DecodeError(other_err)),
		} )
	}.to_rgba();
	let (width, height) = image.dimensions();
	//Derp.
	let mut y = 0;
	let mut rows = Vec::with_capacity(width as usize);
	let mut row = Vec::with_capacity(height as usize);
	for pixel in image.pixels() {
		let pixel_value = (pixel[0], pixel[1], pixel[2], pixel[3]);
		row.push(pixel_value);
		y += 1;
		if y == width {
			y = 0;
			rows.push(row);
			row = Vec::with_capacity(height as usize);
		}
	}
	Ok(rows)
}

