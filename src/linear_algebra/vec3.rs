use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};
use super::{Sqrt, Vec4};

/// A 3D vector.
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Vec3<T: Copy>([T; 3]);

impl<T> Vec3<T> where T: Copy + Mul<Output=T> + Add<Output=T> {
	/// Dot product of two 3D vectors.
	pub fn dot(self, rhs: Self) -> T {
		let l = self.0;
		let r = rhs.0;
		l[0] * r[0] + l[1] * r[1] + l[2] * r[2]
	}
}
impl<T> Vec3<T> where T: Copy + Mul<Output=T> + Sub<Output=T> {
	/// Cross product of two 3D vectors.
	pub fn cross(self, rhs: Self) -> Self {
		let l = self.0;
		let r = rhs.0;
		Vec3( [
			l[1] * r[2] - l[2] * r[1],
			l[2] * r[0] - l[0] * r[2],
			l[0] * r[1] - l[1] * r[0], ] )
	}
}
impl<T> Vec3<T> where T: Copy +
		Add<Output = T> +
		Mul<Output = T> +
		Div<Output = T> +
		Sqrt<Output = T> {
	/// Normalize this 3D vector
	pub fn normalize(self) -> Self {
		let norm = (self[0] * self[0] +
		            self[1] * self[1] +
		            self[2] * self[2]).sqrt();
		Vec3::from([self[0] / norm, self[1] / norm, self[2] / norm])
	}
}

// Arithmetic operations
impl<T> Add for Vec3<T> where T: Copy + Add<Output = T> {
	type Output = Self;
	fn add(self, r: Self) -> Self {
		Vec3([self[0] + r[0], self[1] + r[1], self[2] + r[2]])
	}
}
impl<T> Div<T> for Vec3<T> where T: Copy + Div<Output = T> {
	type Output = Self;
	fn div(self, r: T) -> Self {
		Vec3([self[0] / r, self[1] / r, self[2] / r])
	}
}
impl<T> Mul<T> for Vec3<T> where T: Copy + Mul<Output = T> {
	type Output = Self;
	fn mul(self, r: T) -> Self {
		Vec3([self[0] * r, self[1] * r, self[2] * r])
	}
}
impl<T> Sub for Vec3<T> where T: Copy + Sub<Output = T> {
	type Output = Self;
	fn sub(self, r: Self) -> Self {
		Vec3([self[0] - r[0], self[1] - r[1], self[2] - r[2]])
	}
}

// Indexing and conversion
impl<T: Copy> Index<usize> for Vec3<T> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		&(self.0[index])
	}
}
impl<T: Copy> IndexMut<usize> for Vec3<T> {
	fn index_mut(&mut self, index: usize) -> &mut T {
		&mut (self.0[index])
	}
}
impl<T: Copy> Into<[T; 3]> for Vec3<T> {
	fn into(self) -> [T; 3] {
		self.0
	}
}
impl<T: Copy> From<[T; 3]> for Vec3<T> {
	fn from(other: [T; 3]) -> Self {
		Vec3(other)
	}
}
impl<T: Copy> From<Vec4<T>> for Vec3<T> {
	fn from(other: Vec4<T>) -> Self {
		Vec3([other[0], other[1], other[2]])
	}
}
