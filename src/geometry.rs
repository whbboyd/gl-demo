use glium::{VertexBuffer, IndexBuffer};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: (f32, f32, f32)
}

implement_vertex!(Vertex, position);

#[derive(Copy, Clone, Debug)]
pub struct Normal {
    pub normal: (f32, f32, f32)
}

implement_vertex!(Normal, normal);

#[derive(Debug)]
pub struct Object {
	pub vertices: VertexBuffer<Vertex>,
	pub normals: VertexBuffer<Normal>,
	pub indices: IndexBuffer<u16>,
	pub model_matrix: [[f32; 4]; 4],
	pub material: Material
}

#[derive(Debug, Clone)]
pub struct Material {
	pub light: (f32, f32, f32),
	pub dark: (f32, f32, f32)
}

