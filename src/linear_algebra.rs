//! Linear algebra
use std::ops::{Add, Index, IndexMut, Mul, Sub};

/// A 3D vector.
#[derive(Copy,Clone,Debug)]
pub struct Vec3<T: Copy>([T; 3]);
impl<T> Vec3<T> where T: Copy + Mul<Output=T> + Add<Output=T> {
	/// Dot product of two 3D vectors.
	pub fn dot(self, rhs: Self) -> T {
		let l = self.0;
		let r = rhs.0;
		l[0]*r[0]+l[1]*r[1]+l[2]*r[2]
	}
}
impl<T> Vec3<T> where T: Copy + Mul<Output=T> + Sub<Output=T> {
	/// Cross product of two 3D vectors.
	pub fn cross(self, rhs: Self) -> Self {
		let l = self.0;
		let r = rhs.0;
		Vec3( [
			l[1]*r[2]-l[2]*r[1],
			l[2]*r[0]-l[0]*r[2],
			l[0]*r[1]-l[1]*r[0], ] )
	}
}
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
		let o = other.0;
		Vec3([o[0], o[1], o[2]])
	}
}

/// A 4D vector.
#[derive(Copy,Clone,Debug)]
pub struct Vec4<T: Copy>([T; 4]);
impl<T> Vec4<T> where T: Copy + Mul<Output=T> + Add<Output=T> {
	/// Dot product of two 4D vectors.
	pub fn dot(self, rhs: Self) -> T {
		let l = self.0;
		let r = rhs.0;
		l[0]*r[0]+l[1]*r[1]+l[2]*r[2]+l[3]*r[3]
	}
}
impl<T: Copy> Index<usize> for Vec4<T> {
	type Output = T;
	fn index(&self, index: usize) -> &T {
		&(self.0[index])
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

/// A 3x3 matrix.
#[derive(Copy,Clone,Debug)]
pub struct Mat3<T: Copy>([[T; 3]; 3]);
impl<T> Mul for Mat3<T> where T: Copy + Mul<Output = T> + Add<Output = T> {
	type Output = Self;
	/// Matrix product
	fn mul(self, rhs: Self) -> Self {
		let l = self.0;
		let r = rhs.0;
		let mut result = Mat3([[l[0][0]; 3]; 3]);
		for i in 0..3 {
			for j in 0..3 {
				result.0[i][j] = l[0][i]*r[j][0]+l[1][i]*r[j][1]+l[2][i]*r[j][2];
			}
		}
		result
	}
}
impl<T> Mul<Vec3<T>> for Mat3<T> where T: Copy + Mul<Output = T> + Add<Output=T> {
	type Output = Vec3<T>;
	/// Matrix application
	fn mul(self, rhs: Vec3<T>) -> Vec3<T> {
		let l = self.0;
		let r = rhs.0;
		let mut result = Vec3([l[0][0]; 3]);
		for i in 0..3 {
			result.0[i] = l[0][i]*r[0]+l[1][i]*r[1]+l[2][i]*r[2];
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

/// A 4x4 matrix.
#[derive(Copy,Clone,Debug)]
pub struct Mat4<T: Copy>([[T; 4]; 4]);
impl<T> Mul for Mat4<T> where T: Copy + Mul<Output = T> + Add<Output = T> {
	type Output = Self;
	/// Matrix product
	fn mul(self, rhs: Self) -> Self {
		let l = self.0;
		let r = rhs.0;
		let mut result = Mat4([[l[0][0]; 4]; 4]);
		for i in 0..4 {
			for j in 0..4 {
				result.0[i][j] = l[0][i]*r[j][0]+l[1][i]*r[j][1]+l[2][i]*r[j][2]+l[3][i]*r[j][3];
			}
		}
		result
	}
}
impl<T> Mul<Vec4<T>> for Mat4<T> where T: Copy + Mul<Output = T> + Add<Output=T> {
	type Output = Vec4<T>;
	/// Matrix application
	fn mul(self, rhs: Vec4<T>) -> Vec4<T> {
		let l = self.0;
		let r = rhs.0;
		let mut result = Vec4([l[0][0]; 4]);
		for i in 0..4 {
			result.0[i] = l[0][i]*r[0]+l[1][i]*r[1]+l[2][i]*r[2]+l[3][i]*r[3];
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


