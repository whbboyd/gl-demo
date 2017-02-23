//! Linear algebra
mod mat3;
mod mat4;
mod vec3;
mod vec4;

pub use self::mat3::Mat3;
pub use self::mat4::Mat4;
pub use self::vec3::Vec3;
pub use self::vec4::Vec4;

/// Trait for objects which can have their square root taken
pub trait Sqrt {
	/// The type of the square root
	type Output;
	/// Take the square root
	fn sqrt(self) -> Self::Output;
}
impl Sqrt for f32 {
	type Output = f32;
	fn sqrt(self) -> f32 {
		self.sqrt()
	}
}
impl Sqrt for f64 {
	type Output = f64;
	fn sqrt(self) -> f64 {
		self.sqrt()
	}
}

#[cfg(test)]
mod tests {
	use super::{Mat4, Vec3};

	#[test]
	fn test_mat4_mul() {
		let lhs = Mat4::from([
			[1,  2,  3,  4],
			[5,  6,  7,  8],
			[9,  10, 11, 12],
			[13, 14, 15, 16],
		]);
		let rhs = Mat4::from([
			[17, 18, 19, 20],
			[21, 22, 23, 24],
			[25, 26, 27, 28],
			[29, 30, 31, 32],
		]);
		let expected = Mat4::from([
			[250,  260,  270,  280],
			[618,  644,  670,  696],
			[986,  1028, 1070, 1112],
			[1354, 1412, 1470, 1528],
		]);
		let actual = lhs * rhs;
		assert_eq!(expected, actual);
	}

	#[test]
	fn test_vec3_cross() {
		let lhs = Vec3::from([1, 2, 3]);
		let rhs = Vec3::from([4, 5, 6]);
		let expected = Vec3::from([-3, 6, -3]);
		let actual = lhs.cross(rhs);
		assert_eq!(expected, actual);
	}
}

