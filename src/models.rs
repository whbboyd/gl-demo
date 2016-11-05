use geometry::{Material, Normal, Vertex};

// This is the hardcoded Utah Teapot model from
// https://tomaka.github.io/glium/book/tuto-07-teapot.rs
pub use teapot::VERTICES as teapot_vertices;
pub use teapot::NORMALS as teapot_normals;
pub use teapot::INDICES as teapot_indices;
pub use teapot::MATERIAL as teapot_mat;

pub use models::FLOOR_VERTICES as floor_vertices;
pub use models::FLOOR_NORMALS as floor_normals;
pub use models::FLOOR_INDICES as floor_indices;
pub use models::FLOOR_MATERIAL as floor_mat;

pub const FLOOR_VERTICES: [Vertex; 4] = [
    Vertex { position: (1.0, 0.0, 1.0) },
    Vertex { position: (1.0, 0.0, -1.0) },
    Vertex { position: (-1.0, 0.0, -1.0) },
    Vertex { position: (-1.0, 0.0, 1.0) }
];

pub const FLOOR_NORMALS: [Normal; 4] = [
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) },
    Normal { normal: (0.0, 1.0, 0.0) }
];

pub const FLOOR_INDICES: [u16; 6] = [
	2, 1, 0,
	3, 2, 0
];

pub const FLOOR_MATERIAL: Material = Material {
	light: (0.75, 0.75, 0.75),
	dark: (0.25, 0.25, 0.25),
};

