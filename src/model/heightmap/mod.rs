//! Module for dealing with heightmaps.

/// Simple in-memory heightmap with multiple levels of detail.
pub mod simpleheightmap;

use linear_algebra::Vec3;

/// Minimum functionality for a heightmap.
pub trait Heightmap<'a, T: Copy> {

	/// Get the mesh triangle under a given 3D position, for collision purposes.
	fn get_tri_from_position(&self, pos: &Vec3<T>) -> [Vec3<T>; 3];

	/// Update levels of detail based on the camera's position.
	fn update_lod(&mut self, pos: &Vec3<T>);

}
