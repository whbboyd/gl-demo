//! Objects that have been loaded from disk and cached in system memory.

use errors::*;
use model::{disk, Vertex};
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;

/// Generate the default material to fill in if an object-specific material
/// is not specified or cannot be loaded.
///
/// This generates a magenta diffuse with green specular that should be very
/// eye-catching.
pub fn default_mat() -> Material {
	Material {
		ambient: (0.0, 0.0, 0.0),
		specular: (0.0, 1.0, 0.0),
		texture: vec![vec![(255, 0, 255, 255)]],
	}
}

/// In-memory geometry, that is, `Vertex`s.
#[derive(Debug)]
pub struct Geometry {
	/// The object's vertexes.
	pub vertices: Vec<Vertex>,
	/// The object's geometry, specified by indexes into the vertex vector.
	pub indices: Vec<u16>,
}

/// In-memory material and texture specification.
#[derive(Clone, Debug)]
pub struct Material {
	/// The ambient color. This is multiplied by the texture color in unlit
	/// pixels.
	pub ambient: (f32, f32, f32),
	/// The specular color. This is added to the matte color in the specular
	/// highlight.
	pub specular: (f32, f32, f32),
	/// The texture.
	///
	/// This is a nested `Vec` instead of a `glium::texture::RawImage2D`
	/// because `RawImage2D` lacks needed traits.
	pub texture: Vec<Vec<(u8, u8, u8, u8)>>,
}

/// In-memory model, including geometry and material.
#[derive(Debug)]
pub struct Model {
	/// The object's geometry.
	pub geometry: Rc<Geometry>,
	/// The object's material.
	pub material: Rc<Material>,
}

/// A library of in-memory models.
///
/// This enables sharing of materials between objects and (eventually)
/// management of object lifecycle, loading and caching.
#[derive(Debug)]
pub struct ModelLibrary {
	geoms: RefCell<Vec<Rc<Geometry>>>,
	mats: RefCell<Vec<Rc<Material>>>,
	/// The set of models in this library.
	pub models: RefCell<Vec<Rc<Model>>>,
}

impl ModelLibrary {
	/// Create a new, empty `ModelLibrary`.
	pub fn new() -> ModelLibrary {
		ModelLibrary {
			geoms: RefCell::new(Vec::new()),
			mats: RefCell::new(Vec::new()),
			models: RefCell::new(Vec::new()),
		}
	}

	/// Load a model into this library, and return an `Rc` to the loaded
	/// model.
	pub fn load_model(&self, read: &mut Read) -> Result<Rc<Model>> {
		//TODO While probably correct, this is fantastically inelegant.
		let (geom, mat) = try!{ disk::load_model(read) };
		self.geoms.borrow_mut().push(Rc::new(geom));
		self.mats.borrow_mut().push(Rc::new(mat));
		let model = Rc::new(Model {
			//Because we just pushed these, unwrapping last() is safe.
			geometry: self.geoms.borrow().last().unwrap().clone(),
			material: self.mats.borrow().last().unwrap().clone(),
		});
		self.models.borrow_mut().push(model.clone());
		Ok(model)
	}

	/// Add an existing (already loaded or hardcoded) model into this library,
	/// and return an `Rc` to the loaded model.
	pub fn add_model(&self, geom: Geometry, mat: Material) -> Result<Rc<Model>> {
		self.geoms.borrow_mut().push(Rc::new(geom));
		self.mats.borrow_mut().push(Rc::new(mat));
		let model = Rc::new(Model {
			//Because we just pushed these, unwrapping last() is safe.
			geometry: self.geoms.borrow().last().unwrap().clone(),
			material: self.mats.borrow().last().unwrap().clone(),
		});
		self.models.borrow_mut().push(model.clone());
		Ok(model)
	}
}

