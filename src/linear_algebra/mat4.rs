use std::ops::{Add, Index, IndexMut, Mul};
use super::{Mat3, Vec4};

/// A 4x4 matrix.
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Mat4<T: Copy>([[T; 4]; 4]);
impl<T> Mul for Mat4<T> where T: Copy + Mul<Output = T> + Add<Output = T> {
	type Output = Self;
	/// Matrix product
	fn mul(self, r: Self) -> Self {
		let mut result = Mat4([[self[0][0]; 4]; 4]);
		for i in 0..4 {
			for j in 0..4 {
				result[i][j] = self[i][0] * r[0][j] +
				               self[i][1] * r[1][j] +
				               self[i][2] * r[2][j] +
				               self[i][3] * r[3][j];
			}
		}
		result
	}
}
impl<T> Mul<Vec4<T>> for Mat4<T> where T: Copy + Mul<Output = T> + Add<Output=T> {
	type Output = Vec4<T>;
	/// Matrix application
	fn mul(self, r: Vec4<T>) -> Vec4<T> {
		let mut result = Vec4::from([self[0][0]; 4]);
		for i in 0..4 {
			result[i] = self[i][0] * r[0] +
			            self[i][1] * r[1] +
			            self[i][2] * r[2] +
			            self[i][3] * r[3];
		}
		result
	}
}
impl<T: Copy> Index<usize> for Mat4<T> {
	type Output = [T; 4];
	fn index(&self, index: usize) -> &[T; 4] {
		&(self.0[index])
	}
}
impl<T: Copy> IndexMut<usize> for Mat4<T> {
	fn index_mut(&mut self, index: usize) -> &mut [T; 4] {
		&mut (self.0[index])
	}
}
impl<T: Copy> Into<Mat3<T>> for Mat4<T> {
	fn into(self) -> Mat3<T> {
		Mat3::from([
			[self[0][0], self[0][1], self[0][2]],
			[self[1][0], self[1][1], self[1][2]],
			[self[2][0], self[2][1], self[2][2]],
		])
	}
}
impl<T: Copy> Into<[[T; 4]; 4]> for Mat4<T> {
	fn into(self) -> [[T; 4]; 4] {
		self.0
	}
}
impl<T: Copy> From<[[T; 4]; 4]> for Mat4<T> {
	fn from(other: [[T; 4]; 4]) -> Self {
		Mat4(other)
	}
}
