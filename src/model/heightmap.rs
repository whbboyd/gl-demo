//! Generate heightmap meshes.

use linear_algebra::Vec3;
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
	x_offset: f32,
	z_offset: f32,
	resolution: f32,
}
impl Heightmap {

	/// Create a heightmap at a particular size.
	pub fn with_size(width: usize,
			height: usize,
			x_offset: f32,
			z_offset: f32,
			resolution: f32) -> Heightmap {
		let mut heightmap = Heightmap {
			width: width,
			heights: Vec::with_capacity(width * height),
			x_offset: x_offset,
			z_offset: z_offset,
			resolution: resolution,
		};
		heightmap.heights.resize(width * height, HeightmapVertex { height: 0.0, metadata: () });
		heightmap
	}

	/// Create a heightmap object from a texture
	pub fn from_map(map: &Vec<Vec<(u8, u8, u8, u8)>>,
			lowest: f32,
			highest: f32,
			x_offset: f32,
			z_offset: f32,
			resolution: f32) -> Heightmap {
		let width = map.len();
		let height = map[0].len();
		let mut heightmap = Heightmap::with_size(width, height, x_offset, z_offset, resolution);
		for (x, row) in map.iter().enumerate() {
			for (z, cell) in row.iter().enumerate() {
				let mut height = (cell.0 as f32 + cell.1 as f32 + cell.2 as f32) / 768.0;
				height = height * (highest - lowest) + lowest;
				heightmap.set_height(x, z, height);
			}
		}
		heightmap
	}

	/// Set the height at a particular x/z coordinate.
	pub fn set_height(&mut self, x: usize, y: usize, height: f32) {
		let index = self.get_index(x, y);
		self.heights[index].height = height;
	}

	/// Get the vertex at a particular x/z coordinate.
	pub fn get_vertex(&self, x: usize, z: usize) -> Vertex {
		let index = self.get_index(x, z);

		// Compute the position.
		let position = self.get_position(index);

		// Compute the normal.
		// For all adjacent vertices:
		let adjacents = self.get_adjacent_vertices(x, z);
		let norm = adjacents.len() as f32;
		let mut normal = Vec3::from([0f32; 3]);
		for adj_index in adjacents {
			// Compute the normal to the surface between this vertex and the adjacent
			let adj_pos = self.get_position(adj_index);
			let parallel = position - adj_pos;
			let axis = {
				let xz_norm = f32::sqrt(parallel[0] * parallel[0] + parallel[2] * parallel[2]);
				Vec3::from([parallel[1] / xz_norm, 0.0, parallel[0] / xz_norm])
			};
			let cross = axis.cross(parallel);
			let dot = axis.dot(parallel);
			let adj_normal = cross + (axis * dot);
			adj_normal.normalize();
			// Add them all up
			normal = normal + adj_normal;
		}
		// Normalize
		normal = normal / norm;

		// Texture mapping
		let tex_uv = [position[0], position[2]];

		Vertex {
			position: position.into(),
			normal: normal.into(),
			tex_uv: tex_uv,
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
						indices.push(self.get_index(x, z) as u32);
						indices.push(self.get_index(x, z + 1) as u32);
						indices.push(self.get_index(x + 1, z) as u32);
						// Second triangle:
						indices.push(self.get_index(x + 1, z) as u32);
						indices.push(self.get_index(x, z + 1) as u32);
						indices.push(self.get_index(x + 1, z + 1) as u32);
					} else {
						// First triangle:
						indices.push(self.get_index(x, z) as u32);
						indices.push(self.get_index(x + 1, z + 1) as u32);
						indices.push(self.get_index(x + 1, z) as u32);
						// Second triangle:
						indices.push(self.get_index(x, z) as u32);
						indices.push(self.get_index(x, z + 1) as u32);
						indices.push(self.get_index(x + 1, z + 1) as u32);
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

	/// Get the position in 3D space of a vertex by index.
	pub fn get_position(&self, index: usize) -> Vec3<f32> {
		Vec3::from([
			((index % self.width) as f32 +
				(if (index / self.width) % 2 == 0 { 0.0 } else { 0.5 }))
				* self.resolution + self.x_offset,
			self.heights[index].height,
			(index / self.width) as f32 * ROW_SPACING * self.resolution + self.z_offset,
		])
	}

	fn get_index_from_position(&self, pos: &Vec3<f32>) -> usize {
		let unpos_z = ((pos[2] - self.z_offset) / self.resolution /	ROW_SPACING).floor();
		let unpos_x = ((pos[0] - self.x_offset) / self.resolution -
			(if unpos_z % 2.0 == 0.0 { 0.0 } else { 0.5 } )).floor();
		self.get_index(unpos_x as usize, unpos_z as usize)
	}

	/// Get the triangle under the given position in 3D space
	pub fn get_tri_from_position(&self, pos: &Vec3<f32>) -> [Vec3<f32>; 3] {
		// For reference
		//
		//    A-----B
		//   /|\ 2 /|\
		//  / |1\ /3| \
		// C--k--D--l--E
		//
		// As a precondition, pos must be within the bounds of the heightmap.
		// Behavior is undefined if it's not.
		// TODO: Behave reasonably out of bounds. Floor out of bounds always at -inf?
		let vtx_a = self.get_index_from_position(pos);
		let vtx_a_pos = self.get_position(vtx_a);
		let vtx_a_z = vtx_a / self.width;
		let vtx_a_x = vtx_a % self.width;
		let vtx_d_z = vtx_a_z + 1;
		let vtx_d_x = if vtx_a_z % 2 == 0 { vtx_a_x } else { vtx_a_x + 1};
		let vtx_d = self.get_index(vtx_d_x, vtx_d_z);
		let vtx_d_pos = self.get_position(vtx_d);

		// Case 1 or 2/3: are we below a-d?
		let m = (vtx_d_pos[2] - vtx_a_pos[2]) / (vtx_d_pos[0] - vtx_a_pos[0]);
		let b = vtx_a_pos[2] - m * vtx_a_pos[0];
		if pos[2] > m * pos[0] + b {
			// Case 1
			let vtx_c_pos = self.get_position(vtx_d - 1);
			return [vtx_a_pos, vtx_d_pos, vtx_c_pos];
		} else {
			//Case 2 or 3: are we above b-d?
			let vtx_b_pos = self.get_position(vtx_a + 1);
			let m = (vtx_b_pos[2] - vtx_d_pos[2]) / (vtx_b_pos[0] - vtx_d_pos[0]);
			let b = vtx_b_pos[2] - m * vtx_b_pos[0];
			if pos[2] < m * pos[0] + b {
				// Case 2
				return [vtx_a_pos, vtx_b_pos, vtx_d_pos];
			} else {
				// Case 3
				let vtx_e_pos = self.get_position(vtx_d + 1);
				return [vtx_b_pos, vtx_e_pos, vtx_d_pos];
			}
		}
	}

	/// Get the list of vertices (by index) adjacent to the given vertex.
	fn get_adjacent_vertices(&self, x: usize, z: usize) -> Vec<usize> {
		let mut adjacents = Vec::with_capacity(6);

		// Rows above and below (adjacents 0, 1, 4, 5) depend on row parity.
		if z % 2 == 0 {
			let row_above = z as isize - 1;
			let row_below = z + 1;
			let x_left = x as isize - 1;
			if row_above >= 0 {
				if x_left >= 0 {
					adjacents.push(self.get_index(x_left as usize, row_above as usize));
				}
				adjacents.push(self.get_index(x, row_above as usize));
			}
			if row_below < self.height() {
				if x_left >= 0 {
					adjacents.push(self.get_index(x_left as usize, row_below as usize));
				}
				adjacents.push(self.get_index(x, row_below as usize));
			}
		} else {
			let row_above = z as isize - 1;
			let row_below = z + 1;
			let x_right = x + 1;
			if row_above >= 0 {
				adjacents.push(self.get_index(x, row_above as usize));
				if x_right < self.width {
					adjacents.push(self.get_index(x_right as usize, row_above as usize));
				}
			}
			if row_below < self.height() {
				adjacents.push(self.get_index(x, row_below));
				if x_right < self.width {
					adjacents.push(self.get_index(x_right as usize, row_below as usize));
				}
			}
		}
		let x_left = x as isize - 1;
		let x_right = x + 1;
		if x_left >= 0 {
			adjacents.push(self.get_index(x_left as usize, z));
		}
		if x_right < self.width {
			adjacents.push(self.get_index(x_right as usize, z));
		}
 
		adjacents
	}

}

#[cfg(test)]
mod tests {
	use super::Heightmap;
	use linear_algebra::Vec3;

	#[test]
	fn test_adjacents() {
		// 0---1---2---3
		//  \ / \ / \ / \
		//   4---5---6---7
		//  / \ / \ / \ /
		// 8---9---10--11
		//  \ / \ / \ / \
		//   12--13--14--15

		let map = Heightmap::with_size(4, 4, 0.0, 0.0, 1.0);

		// Top left: index 0
		let expected = vec![4, 1];
		let actual = map.get_adjacent_vertices(0, 0);
		assert_eq!(expected, actual);

		// Top: index 1
		let expected = vec![4, 5, 0, 2];
		let actual = map.get_adjacent_vertices(1, 0);
		assert_eq!(expected, actual);

		// Top right: index 3
		let expected = vec![6 ,7, 2];
		let actual = map.get_adjacent_vertices(3, 0);
		assert_eq!(expected, actual);

		// Left, odd row: index 4
		let expected = vec![0, 1, 8, 9, 5];
		let actual = map.get_adjacent_vertices(0, 1);
		assert_eq!(expected, actual);

		// Middle, odd row: index 5
		let expected = vec![1, 2, 9, 10, 4, 6];
		let actual = map.get_adjacent_vertices(1, 1);
		assert_eq!(expected, actual);

		// Right, odd row: index 7
		let expected = vec![3, 11, 6];
		let actual = map.get_adjacent_vertices(3, 1);
		assert_eq!(expected, actual);

		// Left, even row: index 8
		let expected = vec![4, 12, 9];
		let actual = map.get_adjacent_vertices(0, 2);
		assert_eq!(expected, actual);

		// Middle, even row: index 10
		let expected = vec![5, 6, 13, 14, 9, 11];
		let actual = map.get_adjacent_vertices(2, 2);
		assert_eq!(expected, actual);

		// Right, even row: index 11
		let expected = vec![6, 7, 14, 15, 10];
		let actual = map.get_adjacent_vertices(3, 2);
		assert_eq!(expected, actual);

		// Bottom left, odd row: index 12
		let expected = vec![8, 9, 13];
		let actual = map.get_adjacent_vertices(0, 3);
		assert_eq!(expected, actual);

		// Bottom, odd row: index 14
		let expected = vec![10, 11, 13, 15];
		let actual = map.get_adjacent_vertices(2, 3);
		assert_eq!(expected, actual);

		// Bottom right, odd row: index 15
		let expected = vec![11, 14];
		let actual = map.get_adjacent_vertices(3, 3);
		assert_eq!(expected, actual);

		// For even bottom rows
		let map = Heightmap::with_size(4, 3, 0.0, 0.0, 1.0);

		// Bottom left, even row: index 8
		let expected = vec![4, 9];
		let actual = map.get_adjacent_vertices(0, 2);
		assert_eq!(expected, actual);

		// Bottom, even row: index 10
		let expected = vec![5, 6, 9, 11];
		let actual = map.get_adjacent_vertices(2, 2);
		assert_eq!(expected, actual);

		// Bottom right, even row: index 11
		let expected = vec![6, 7, 10];
		let actual = map.get_adjacent_vertices(3, 2);
		assert_eq!(expected, actual);

	}

	#[test]
	fn test_get_index_from_position() {
		let map = Heightmap::with_size(4, 4, 0.0, 0.0, 1.0);

		for index in 0..16 {
			let pos = map.get_position(index);
			let unpos = map.get_index_from_position(&pos);
			assert_eq!(index, unpos,
				"({}, {}): expected {}, got {}", pos[0], pos[2], index, unpos);
		}

		let pos = Vec3::from([0.5, 0.0, 0.5]);
		assert_eq!(0, map.get_index_from_position(&pos));
		let pos = Vec3::from([1.49, 0.0, 1.0]);
		assert_eq!(4, map.get_index_from_position(&pos));
		let pos = Vec3::from([1.51, 0.0, 1.0]);
		assert_eq!(5, map.get_index_from_position(&pos));
		
	}
}
