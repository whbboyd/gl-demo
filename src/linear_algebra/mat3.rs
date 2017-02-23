use std::ops::{Add, Index, IndexMut, Mul};
use super::Vec3;

/// A 3x3 matrix.
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Mat3<T: Copy>([[T; 3]; 3]);
impl<T> Mul for Mat3<T> where T: Copy + Mul<Output = T> + Add<Output = T> {
	type Output = Self;
	/// Matrix product
	fn mul(self, r: Self) -> Self {
		let mut result = Mat3([[self[0][0]; 3]; 3]);
		for i in 0..3 {
			for j in 0..3 {
				result[i][j] = self[i][0] * r[0][j] +
				               self[i][1] * r[1][j] +
				               self[i][2] * r[2][j];
			}
		}
		result
	}
}
impl<T> Mul<Vec3<T>> for Mat3<T> where T: Copy + Mul<Output = T> + Add<Output=T> {
	type Output = Vec3<T>;
	/// Matrix application
	fn mul(self, r: Vec3<T>) -> Vec3<T> {
		let mut result = Vec3::from([self[0][0]; 3]);
		for i in 0..3 {
			result[i] = self[i][0] * r[0] +
			            self[i][1] * r[1] +
			            self[i][2] * r[2];
		}
		result
	}
}
impl<T: Copy> Index<usize> for Mat3<T> {
	type Output = [T; 3];
	fn index(&self, index: usize) -> &[T; 3] {
		&(self.0[index])
	}
}
impl<T: Copy> IndexMut<usize> for Mat3<T> {
	fn index_mut(&mut self, index: usize) -> &mut [T; 3] {
		&mut (self.0[index])
	}
}
impl<T: Copy> Into<[[T; 3]; 3]> for Mat3<T> {
	fn into(self) -> [[T; 3]; 3] {
		self.0
	}
}
impl<T: Copy> From<[[T; 3]; 3]> for Mat3<T> {
	fn from(other: [[T; 3]; 3]) -> Self {
		Mat3(other)
	}
}

