//! Logic for handling models.
//!
//! This includes structs and methods to load them from disk, cache them in
//! system memory, and upload them to GPU memory.

pub mod disk;
pub mod gpu;
pub mod heightmap;
pub mod mem;

/// A vertex and associated data.
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
	/// The location of this vertex.
    pub position: [f32; 3],
	/// The normal corresponding to this vertex.
	pub normal: [f32; 3],
	/// The texture UV coordinates at this vertex.
	pub tex_uv: [f32; 2],
}
implement_vertex!(Vertex, position, normal, tex_uv);

