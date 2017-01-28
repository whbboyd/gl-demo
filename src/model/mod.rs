pub mod disk;
pub mod gpu;
pub mod mem;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: (f32, f32, f32),
	pub normal: (f32, f32, f32),
	pub tex_uv: (f32, f32),
}
implement_vertex!(Vertex, position, normal, tex_uv);

