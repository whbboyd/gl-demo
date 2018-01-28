//! A library of available models, to handle caching and converting

use errors::*;
use model::{disk, Vertex};
use model::mem::{Geometry, Material, Model};
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;

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

