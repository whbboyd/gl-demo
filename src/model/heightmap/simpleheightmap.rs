use glium::backend::Facade;
use glium::Frame;
use linear_algebra::{Mat4, Vec3};
use model::{gpu, mem, Vertex};
use model::heightmap::Heightmap;
use renderable::{DefaultRenderState, Renderable};
use std::cmp::min;
use std::f32;
use std::rc::Rc;
use std::usize;
use time::PreciseTime;

/// The spacing between rows of a mesh of equilateral triangles with sides of
/// length one. This is equal to 0.5 * tan(pi / 3).
const ROW_SPACING: f32 = 0.8660254037844386;

#[derive(Copy, Clone, Debug)]
struct HeightmapVertex {
	height: f32,
	metadata: (),
}

struct LoDTile {
	lod: usize,
	tile: gpu::Model,
}

/// A heightmap, with high-resolution geometry stored entirely in-memory.
pub struct SimpleHeightmap<'a> {
	geometry: SimpleHeightmapGeometry,
	display: &'a Facade,
	material: Rc<mem::Material>,
	lods: Vec<LoDTile>,
	tile_size: usize,
	lod_zone: (f32, f32),
}

impl<'a> Heightmap<'a, f32> for SimpleHeightmap<'a> {

	/// Get the triangle under the given position in 3D space
	fn get_tri_from_position(&self, pos: &Vec3<f32>) -> [Vec3<f32>; 3] {
		let g = &self.geometry;
		// If we're not over the heightmap, collide at -infinity
		//TODO: These bounds could certainly be tighter, but it's not likely to matter
		if pos[0] < g.x_offset  + 1.0 * g.resolution ||
				pos[0] > g.x_offset + (g.width as f32 - 1.0) * g.resolution ||
				pos[2] < g.z_offset + 1.0 * g.resolution ||
				pos[2] > g.z_offset + (g.height() as f32 - 1.0) * g.resolution * ROW_SPACING {
			return [Vec3::from([0.0, f32::NEG_INFINITY, 0.0]),
					Vec3::from([1.0, f32::NEG_INFINITY, 0.0]),
					Vec3::from([0.0, f32::NEG_INFINITY, 1.0])];
		}

		// For reference
		//
		//    A-----B
		//   /|\ 2 /|\
		//  / |1\ /3| \
		// C--k--D--l--E
		//
		let vtx_a = g.get_index_from_position(pos);
		let vtx_a_pos = g.get_position(vtx_a);
		let vtx_a_z = vtx_a / g.width;
		let vtx_a_x = vtx_a % g.width;
		let vtx_d_z = vtx_a_z + 1;
		let vtx_d_x = if vtx_a_z % 2 == 0 { vtx_a_x } else { vtx_a_x + 1};
		let vtx_d = g.get_index(vtx_d_x, vtx_d_z);
		let vtx_d_pos = g.get_position(vtx_d);

		// Case 1 or 2/3: are we below A-D?
		let m = (vtx_d_pos[2] - vtx_a_pos[2]) / (vtx_d_pos[0] - vtx_a_pos[0]);
		let b = vtx_a_pos[2] - m * vtx_a_pos[0];
		if pos[2] > m * pos[0] + b {
			// Case 1
			let vtx_c_pos = g.get_position(vtx_d - 1);
			return [vtx_a_pos, vtx_d_pos, vtx_c_pos];
		} else {
			//Case 2 or 3: are we above B-D?
			let vtx_b_pos = g.get_position(vtx_a + 1);
			let m = (vtx_b_pos[2] - vtx_d_pos[2]) / (vtx_b_pos[0] - vtx_d_pos[0]);
			let b = vtx_b_pos[2] - m * vtx_b_pos[0];
			if pos[2] < m * pos[0] + b {
				// Case 2
				return [vtx_a_pos, vtx_b_pos, vtx_d_pos];
			} else {
				// Case 3
				let vtx_e_pos = g.get_position(vtx_d + 1);
				return [vtx_b_pos, vtx_e_pos, vtx_d_pos];
			}
		}
	}

	/// Update the GPU geometry to account for changing level of detail with location.
	fn update_lod(&mut self, pos: &Vec3<f32>) {
		// Compute LoD zone under pos
		let lod_zone_size = self.tile_size as f32 * self.geometry.resolution;
		let diff = ((pos[0] - self.lod_zone.0).abs(), (pos[2] - self.lod_zone.1).abs());
		// If both dimensions aren't within bounds of the LoD zone, recompute.
		// This conditional is phrased this way so regeneration triggers when
		// lod_zone has NaN dimensions.
		if !(diff.0 <= lod_zone_size && diff.1 <= lod_zone_size) {
			let start_time = PreciseTime::now();
			let mut updated_tiles = 0usize;
			// Update LoD zone 
			let new_lod_zone = (pos[0] - (pos[0] % (lod_zone_size / 2.0)),
				pos[2] - (pos[2] % (lod_zone_size / 2.0)));
			//TODO: Range.step_by is recent and unstable.
			let mut x = 0;
			let mut index = 0;
			while x < self.geometry.width {
				let mut z = 0;
				while z < self.geometry.height() {
					let lod = gen_lod(&self, pos, x, z);
					if lod != self.lods[index].lod {
						let top_z = z;
						let left_x = x;
						let bottom_z = z + self.tile_size;
						let right_x = x + self.tile_size;
						//TODO: Cache these somewhere so we can fetch them
						// instead of regenerating.
						self.lods[index] = LoDTile {
							lod: lod,
							tile: gpu::Model::from_mem(self.display,
								&mem::Model {
									geometry: Rc::new(self.geometry.as_geometry(
											lod, left_x, top_z, right_x, bottom_z)),
									material: self.material.clone(),
								}).unwrap()
							};
						updated_tiles += 1;
					}
					z += self.tile_size;
					index += 1;
				}
				x += self.tile_size;
			}
			self.lod_zone = new_lod_zone;
			info!("Updated LoD ({} of {} tiles) in {:?} milliseconds",
					updated_tiles,
					self.lods.len(),
					start_time.to(PreciseTime::now()).num_milliseconds());
		}
	}

}

fn gen_lod(hm: &SimpleHeightmap, pos: &Vec3<f32>, x: usize, z: usize) -> usize {
	// Compute tile center
	let center_x = (x as f32 + hm.tile_size as f32 / 2.0) *
			hm.geometry.resolution + hm.geometry.x_offset;
	let center_z = (z as f32 + hm.tile_size as f32 / 2.0) *
			hm.geometry.resolution * ROW_SPACING + hm.geometry.z_offset;

	// Compute distance between location and center
	let distance_square = (pos[0] - center_x) * (pos[0] - center_x) +
			(pos[2] - center_z) * (pos[2] - center_z);
	let tile_distance_square = distance_square / 
			(hm.tile_size as f32 * hm.geometry.resolution *
			hm.tile_size as f32 * hm.geometry.resolution);

	// This is the greatest power of two less than distance_square
	min(f32::max(1.0, tile_distance_square.log(2.0).floor().exp2()) as usize,
			hm.tile_size - 1)
}

impl<'a, 'b> Renderable<&'a DefaultRenderState<'a>, &'a mut Frame> for SimpleHeightmap<'b> {
	fn render(&self, renderstate: &'a DefaultRenderState, target: &mut Frame) {
		for model in self.lods.iter() {
			gpu::ModelInstance {
				model: &model.tile,
				model_matrix: Mat4::from( [
					[1.0,		0.0,	0.0,	0.0],
					[0.0,		1.0,	0.0,	0.0],
					[0.0,		0.0,	1.0,	0.0],
					[0.0,		0.0,	0.0,	1.0] ], ) }
				.render(renderstate, target)
			// Draw LoD HuD in center of tile
		}
	}
}

impl<'a> SimpleHeightmap<'a> {

	/// Create a heightmap at a particular size.
	pub fn with_size(width: usize,
			height: usize,
			x_offset: f32,
			z_offset: f32,
			resolution: f32,
			display: &'a Facade,
			material: mem::Material) -> SimpleHeightmap<'a> {
		let mut heightmap = SimpleHeightmap {
			geometry: SimpleHeightmapGeometry {
				width: width,
				heights: Vec::with_capacity(width * height),
				x_offset: x_offset,
				z_offset: z_offset,
				resolution: resolution, },
			display: display,
			material: Rc::new(material),
			lods: Vec::with_capacity(width * height),
			tile_size: 256, //FIXME: Probably shouldn't be hardcoded.
			lod_zone: (f32::NAN, f32::NAN),
		};
		let init_geom = Rc::new(mem::Geometry { vertices: Vec::new(), indices: Vec::new() } );
		for _ in 0..(width * height) / (heightmap.tile_size * heightmap.tile_size) {
			heightmap.lods.push(LoDTile {
					lod: usize::MAX,
					tile: gpu::Model::from_mem(
						heightmap.display,
						&mem::Model {
							geometry: init_geom.clone(),
							material: heightmap.material.clone(),
						}).unwrap()
					} );
		}
		heightmap.geometry.heights.resize(
				width * height,
				HeightmapVertex { height: 0.0, metadata: ()
			});

		heightmap
	}

	/// Create a heightmap object from a texture
	pub fn from_map(map: &Vec<Vec<(u8, u8, u8, u8)>>,
			lowest: f32,
			highest: f32,
			x_offset: f32,
			z_offset: f32,
			resolution: f32,
			display: &'a Facade,
			material: mem::Material) -> SimpleHeightmap<'a> {
		let width = map.len();
		let height = map[0].len();
		let mut heightmap = SimpleHeightmap::with_size(
				width, height, x_offset, z_offset, resolution, display, material);
		for (x, row) in map.iter().enumerate() {
			for (z, cell) in row.iter().enumerate() {
				let mut height = (cell.0 as f32 + cell.1 as f32 + cell.2 as f32) / 768.0;
				height = height * (highest - lowest) + lowest;
				heightmap.geometry.set_height(x, z, height);
			}
		}
		heightmap
	}

}

struct SimpleHeightmapGeometry {
	width: usize,
	heights: Vec<HeightmapVertex>,
	x_offset: f32,
	z_offset: f32,
	resolution: f32,
}

impl SimpleHeightmapGeometry {

	/// Set the height at a particular x/z coordinate.
	fn set_height(&mut self, x: usize, y: usize, height: f32) {
		let index = self.get_index(x, y);
		self.heights[index].height = height;
	}

	/// Get the vertex at a particular x/z coordinate.
	fn get_vertex(&self, x: usize, z: usize) -> Vertex {
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
	fn height(&self) -> usize {
		self.heights.len() / self.width
	}

	/// Convert this heightmap to in-memory 3D geometry.
	fn as_geometry(&self,
	               lod: usize,
				   left_x: usize,
				   top_z: usize,
				   right_x: usize,
				   bottom_z: usize) -> mem::Geometry {
		let left_x = min(left_x, self.width);
		let top_z = min(top_z, self.height());
		let right_x = min(right_x, self.width);
		let bottom_z = min(bottom_z, self.height());
		let width = (right_x - left_x) / lod;
		let height = (bottom_z - top_z) / lod;
		let mut vertices = Vec::with_capacity(width * height);
		let mut indices = Vec::new();
		let mut z = top_z;
		let mut idx_z = 0;
		while z < bottom_z {
			let mut x = left_x;
			let mut idx_x = 0;
			while x < right_x {
				vertices.push(self.get_vertex(x, z));
				// Compute indices
				//TODO: If the tile dimensions are not evenly divisible by the
				// LoD, this will generate out-of-bounds indices.
				if x < right_x - lod && z < bottom_z - lod {
					if z % 2 == 0 {
						// First triangle:
						indices.push((idx_z + idx_x * width) as u16);
						indices.push((idx_z + (idx_x + 1) * width) as u16);
						indices.push((idx_z + 1 + idx_x * width) as u16);
						// Second triangle:
						indices.push((idx_z + 1 + idx_x * width) as u16);
						indices.push((idx_z + (idx_x + 1) * width) as u16);
						indices.push((idx_z + 1 + (idx_x + 1) * width) as u16);
					} else {
						// First triangle:
						indices.push((idx_z + idx_x * width) as u16);
						indices.push((idx_z + 1 + (idx_x + 1) * width) as u16);
						indices.push((idx_z + 1 + idx_x * width) as u16);
						// Second triangle:
						indices.push((idx_z + idx_x * width) as u16);
						indices.push((idx_z + (idx_x + 1) * width) as u16);
						indices.push((idx_z + 1 + (idx_x + 1) * width) as u16);
					}
				}
				x += lod;
				idx_x += 1;
			}
			z += lod;
			idx_z += 1;
		}

		let vs = vertices.len();
		let mi = indices.iter().max().unwrap_or(&0).clone() as usize;
		if mi != vs || vs > u16::max_value() as usize + 1 {
			error!("LoD vertices and indices mismatch for tile {},{}-{},{}: \
					vertices: {}, max index: {}",
					left_x, top_z, right_x, bottom_z, vs, mi);
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
	fn get_position(&self, index: usize) -> Vec3<f32> {
		Vec3::from([
			((index % self.width) as f32 +
				(if (index / self.width) % 2 == 0 { 0.0 } else { 0.5 })) * self.resolution +
				self.x_offset,
			self.heights[index].height,
			(index / self.width) as f32 * ROW_SPACING * self.resolution + self.z_offset,
		])
	}

	/// Get the index of the nearest vertex north and west of the given position.
	fn get_index_from_position(&self, pos: &Vec3<f32>) -> usize {
		let unpos_z = ((pos[2] - self.z_offset) / self.resolution /	ROW_SPACING).floor();
		let unpos_x = ((pos[0] - self.x_offset) / self.resolution -
			(if unpos_z % 2.0 == 0.0 { 0.0 } else { 0.5 } )).floor();
		self.get_index(unpos_x as usize, unpos_z as usize)
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
	use super::SimpleHeightmapGeometry;
	use super::HeightmapVertex;
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

		let mut map = SimpleHeightmapGeometry {
				width: 4,
				heights: Vec::with_capacity(4 * 4),
				x_offset: 0.0,
				z_offset: 0.0,
				resolution: 1.0, };
		map.heights.resize(
				4 * 4,
				HeightmapVertex { height: 0.0, metadata: () });

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
		let mut map = SimpleHeightmapGeometry {
				width: 4,
				heights: Vec::with_capacity(4 * 3),
				x_offset: 0.0,
				z_offset: 0.0,
				resolution: 1.0, };
		map.heights.resize(
				4 * 3,
				HeightmapVertex { height: 0.0, metadata: () });

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
		let mut map = SimpleHeightmapGeometry {
				width: 4,
				heights: Vec::with_capacity(4 * 4),
				x_offset: 0.0,
				z_offset: 0.0,
				resolution: 1.0, };
		map.heights.resize(
				4 * 4,
				HeightmapVertex { height: 0.0, metadata: () });

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
