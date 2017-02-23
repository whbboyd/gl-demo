use std::ops::{Add, Index, IndexMut, Mul};

/// A 4D vector.
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Vec4<T: Copy>([T; 4]);
impl<T> Vec4<T> where T: Copy + Mul<Output=T> + Add<Output=T> {
	/// Dot product of two 4D vectors.
	pub fn dot(self, rhs: Self) -> T {
		let l = self.0;
		let r = rhs.0;
		l[0] * r[0] + l[1] * r[1] + l[2] * r[2] + l[3] * r[3]
	}
}
impl<T: Copy> Index<usize> for Vec4<T> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		&(self.0[index])
	}
}
impl<T: Copy> IndexMut<usize> for Vec4<T> {
	fn index_mut(&mut self, index: usize) -> &mut T {
		&mut (self.0[index])
	}
}
impl<T: Copy> Into<[T; 4]> for Vec4<T> {
	fn into(self) -> [T; 4] {
		self.0
	}
}
impl<T: Copy> From<[T; 4]> for Vec4<T> {
	fn from(other: [T; 4]) -> Self {
		Vec4(other)
	}
}
