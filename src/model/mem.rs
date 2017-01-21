use model::{disk, Normal, Vertex};
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;

// Magenta with green specular
pub const DEFAULT_MAT: Material = Material {
	ambient: (0.0, 0.0, 0.0),
	diffuse: (1.0, 0.0, 1.0),
	specular: (0.0, 1.0, 0.0),
	texture: None,
};

#[derive(Debug)]
pub struct Geometry {
	pub vertices: Vec<Vertex>,
	pub normals: Vec<Normal>,
	//TODO: u16 limits model complexity fairly significantly. Will this cause problems?
	pub indices: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct Material {
	pub ambient: (f32, f32, f32),
	pub diffuse: (f32, f32, f32),
	pub specular: (f32, f32, f32),
	pub texture: Option<Vec<Vec<u32>>>,
}

#[derive(Debug)]
pub struct Model {
	pub geometry: Rc<Geometry>,
	pub material: Rc<Material>,
}

#[derive(Debug)]
pub struct ModelLibrary {
	geoms: RefCell<Vec<Rc<Geometry>>>,
	mats: RefCell<Vec<Rc<Material>>>,
	pub models: RefCell<Vec<Rc<Model>>>,
}

impl ModelLibrary {
	pub fn new() -> ModelLibrary {
		ModelLibrary {
			geoms: RefCell::new(Vec::new()),
			mats: RefCell::new(Vec::new()),
			models: RefCell::new(Vec::new()),
		}
	}

	pub fn load_model(&self, read: &mut Read) -> Rc<Model> {
		//TODO While probably correct, this is fantastically inelegant.
		let (geom, mat) = disk::load_model(read).unwrap();
		self.geoms.borrow_mut().push(Rc::new(geom));
		self.mats.borrow_mut().push(Rc::new(mat));
		let model = Rc::new(Model {
			geometry: self.geoms.borrow().last().unwrap().clone(),
			material: self.mats.borrow().last().unwrap().clone(),
		});
		self.models.borrow_mut().push(model.clone());
		model
	}
}

