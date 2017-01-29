//! Logic for handling models.
//!
//! This includes structs and methods to load them from disk, cache them in
//! system memory, and upload them to GPU memory.

pub mod disk;
pub mod gpu;
pub mod mem;

/// A vertex and associated data.
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
	/// The location of this vertex.
    pub position: (f32, f32, f32),
	/// The normal corresponding to this vertex.
	pub normal: (f32, f32, f32),
	/// The texture UV coordinates at this vertex.
	pub tex_uv: (f32, f32),
}
implement_vertex!(Vertex, position, normal, tex_uv);

