
// This is the hardcoded Utah Teapot model from
// https://tomaka.github.io/glium/book/tuto-07-teapot.rs
pub use teapot::VERTICES as teapot_vertices;
pub use teapot::NORMALS as teapot_normals;
pub use teapot::INDICES as teapot_indices;
pub use teapot::MATERIAL as teapot_mat;

use geometry::{Material, Normal, Vertex};

pub const floor_vertices: [Vertex; 4] = [
    Vertex { position: (1.0, 0.0, 1.0) },
    Vertex { position: (1.0, 0.0, -1.0) },
    Vertex { position: (-1.0, 0.0, -1.0) },
    Vertex { position: (-1.0, 0.0, 1.0) }
];

pub const floor_normals: [Normal; 4] = [
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) }
];

pub const floor_indices: [u16; 6] = [
	2, 1, 0,
	3, 2, 0
];

pub const floor_mat: Material = Material {
	light: (0.75, 0.75, 0.75),
	dark: (0.25, 0.25, 0.25),
};

