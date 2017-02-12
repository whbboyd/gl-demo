//! Generate heightmap meshes.

use errors::*;
use model::{mem, Vertex};
use std::f32;

/// The spacing between rows of a mesh of equilateral triangles with sides of
/// length one. This is equal to 0.5 * tan(pi / 3).
const ROW_SPACING: f32 = 0.8660254037844386;

#[derive(Copy, Clone, Debug)]
struct HeightmapVertex {
	height: f32,
	metadata: (),
}

/// A heightmap.
#[derive(Debug)]
pub struct Heightmap {
	width: usize,
	heights: Vec<HeightmapVertex>,
	scale: f32,
}
impl Heightmap {

	/// Create a heightmap at a particular size.
	pub fn with_size(width: usize, height: usize, scale: f32) -> Heightmap {
		let mut heightmap = Heightmap {
			width: width,
			heights: Vec::with_capacity(width * height),
			scale: scale,
		};
		heightmap.heights.resize(width * height, HeightmapVertex { height: 0.0, metadata: () });
		heightmap
	}

	/// Get the vertex at a particular x/z coordinate.
	pub fn get_vertex(&self, x: usize, z: usize) -> Vertex {
		let index = self.get_index(x, z);

		// Compute the position.
		let position = [
			((index % self.width) as f32 +
				(if (index / self.width) % 2 == 0 { 0.0 } else { 0.5 }))
				* self.scale,
			self.heights[index].height * self.scale,
			(index / self.width) as f32 * ROW_SPACING * self.scale,
		];

		// Compute the normal.
		// TODO: make this more better
		// For all adjacent vertices:
		let adjacents = self.get_adjacent_vertices(x, z);
		let norm = adjacents.len() as f32;
		let mut normal = [0f32, 0f32, 0f32];
		for index in adjacents {
			let adj_pos = [
				((index % self.width) as f32 +
					(if (index / self.width) % 2 == 0 { 0.0 } else { 0.5 }))
					* self.scale,
				self.heights[index].height * self.scale,
				(index / self.width) as f32 * ROW_SPACING * self.scale,
			];
			let parallel = [position[0] - adj_pos[0],
				position[1] - adj_pos[1],
				position[2] - adj_pos[2]];
			let adj_normal = if parallel[1] > 0.0 {
				[-parallel[0], 1.0 / parallel[1], -parallel[2]]
			} else if parallel[1] < 0.0 {
				[parallel[0], -1.0 / parallel[1], parallel[2]]
			} else {
				[0.0, 1.0, 0.0]
			};
			let adj_normal_len = f32::sqrt(adj_normal[0] * adj_normal[0] +
				adj_normal[1] * adj_normal[1] + adj_normal[2] * adj_normal[2]);
			normal[0] += adj_normal[0] / adj_normal_len;
			normal[1] += adj_normal[1] / adj_normal_len;
			normal[2] += adj_normal[2] / adj_normal_len;
		}
		normal = [normal[0] / norm, normal[1] / norm, normal[2] / norm];
		Vertex {
			position: position,
			normal: normal,
			// TODO: generate texture mapping
			tex_uv: [0.0, 0.0],
		}
	}

	/// Get the height in rows of this Heightmap.
	pub fn height(&self) -> usize {
		self.heights.len() / self.width
	}

	/// Convert this heightmap to in-memory 3D geometry.
	pub fn as_geometry(&self) -> mem::Geometry {
		let mut vertices = Vec::with_capacity(self.heights.len());
		let mut indices = Vec::new();
		for z in 0..self.height() {
			for x in 0..self.width {
				vertices.push(self.get_vertex(x, z));
				// Compute indices
				if x < self.width - 1 && z < self.height() - 1 {
					if z % 2 == 0 {
						// First triangle:
						indices.push(self.get_index(x, z) as u16);
						indices.push(self.get_index(x, z + 1) as u16);
						indices.push(self.get_index(x + 1, z) as u16);
						// Second triangle:
						indices.push(self.get_index(x + 1, z) as u16);
						indices.push(self.get_index(x, z + 1) as u16);
						indices.push(self.get_index(x + 1, z + 1) as u16);
					} else {
						// First triangle:
						indices.push(self.get_index(x, z) as u16);
						indices.push(self.get_index(x + 1, z + 1) as u16);
						indices.push(self.get_index(x + 1, z) as u16);
						// Second triangle:
						indices.push(self.get_index(x, z) as u16);
						indices.push(self.get_index(x, z + 1) as u16);
						indices.push(self.get_index(x + 1, z + 1) as u16);
					}
				}
			}
		}

		mem::Geometry {
			vertices: vertices,
			indices: indices,
		}
	}

	/// Get the index into the heights vector from an x/z coordinate pair.
	fn get_index(&self, x: usize, z: usize) -> usize {
		x + z * self.width
	}

	/// Get the list of vertices (by index) adjacent to the given vertex.
	fn get_adjacent_vertices(&self, x: usize, z: usize) -> Vec<usize> {
		let mut adjacents = Vec::with_capacity(6);

		// Rows above and below (adjacents 0, 1, 4, 5) depend on row parity.
		if x % 2 == 0 {
			let row_above = x as isize - 1;
			let row_below = x + 1;
			let z_left = z as isize - 1;
			if row_above >= 0 {
				if z_left >= 0 {
					adjacents.push(self.get_index(row_above as usize, z_left as usize));
				}
				adjacents.push(self.get_index(row_above as usize, z));
			}
			if row_below < self.height() {
				if z_left >= 0 {
					adjacents.push(self.get_index(row_below as usize, z_left as usize));
				}
				adjacents.push(self.get_index(row_below, z));
			}
		} else {
			let row_above = x as isize - 1;
			let row_below = x + 1;
			let z_right = z + 1;
			if row_above >= 0 {
				adjacents.push(self.get_index(row_above as usize, z));
				if z_right < self.width {
					adjacents.push(self.get_index(row_above as usize, z_right));
				}
			}
			if row_below < self.height() {
				adjacents.push(self.get_index(row_below, z));
				if z_right < self.width {
					adjacents.push(self.get_index(row_below as usize, z_right));
				}
			}
		}
		let z_left = z as isize - 1;
		let z_right = z + 1;
		if z_left >= 0 {
			adjacents.push(self.get_index(x, z_left as usize));
		}
		if z_right < self.width {
			adjacents.push(self.get_index(x, z_right));
		}
 
		adjacents
	}

}

