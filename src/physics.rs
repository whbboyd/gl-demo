//! Module to handle world physics.
//!
//! Right now, this is just character movement and gravity.

use MovementState;
use linear_algebra::Vec3;


/// A character's physical state.
///
/// This includes location and velocity, as well as relevant constants like
/// maximum XZ movement speed, XZ deceleration due to friction, maximum jump
/// speed, and acceleration due to gravity.
#[derive(Clone, Copy, Debug)]
pub struct CharacterState {
	loc: Vec3<f32>,
	vel: Vec3<f32>,
	max_speed: f32,
	decel: f32,
	max_jump: f32,
	gravity: f32
}
impl CharacterState {
	/// Create a new CharacterState.
	///
	///  * `loc`: The location of this character.
	///  * `vel`: The velocity of this character, typically
	///		`[0.0, 0.0, 0.0f32]` initially.
	///  * `max_speed`: The maximum speed, in units/frame, this character can
	///		achieve on the XZ plane.
	///  * `decel`: The rate, in units/frame^2, at which this character
	///		decelerates due to friction in the absence of movement input.
	///  * `max_jump`: The maximum speed, in units/frame, this character can
	///		achieve on the Y axis while jumping.
	///  * `gravity`: The acceleration, in units/frame^2, this character
	///		experiences downward on the Y axis due to gravity. Note that this
	///		value should be positive.
	pub fn new(loc: Vec3<f32>,
			vel: Vec3<f32>,
			max_speed: f32,
			decel: f32,
			max_jump: f32,
			gravity: f32) -> CharacterState {
	CharacterState {
		loc: loc,
		vel: vel,
		max_speed: max_speed,
		decel: decel,
		max_jump: max_jump,
		gravity: gravity}
	}

	/// Update the character's location and velocity based on inputs, gravity and
	/// friction.
	///
	/// This does all of the following:
	///
	///  * Accelerates the character on the XZ plane according to movement inputs.
	///		Acceleration takes five frames to reach maximum speed.
	///  * Decelerates the character on the XZ plane according to friction
	///		(`CharacterState.decel`).
	///  * Handle jump acceleration and timeout. Jumping takes five frames to
	///		reach maximum speed.
	///  * Apply static gravitational acceleration.
	///  * Clamp Y location above zero for floor clipping.
	pub fn do_char_movement(&mut self, dir: &Vec3<f32>, movement: &mut MovementState,
			/*XXX*/ heightmap: &::model::heightmap::Heightmap<f32> ) {

		// Figure out ground height at our location
		let hm_vertices = heightmap.get_tri_from_position(&self.loc);
		let hm_normal = (hm_vertices[0] - hm_vertices[2])
				.cross(hm_vertices[0] - hm_vertices[1]);
		let hm_d = hm_normal.dot(hm_vertices[0]);
		let height = (hm_d -
				hm_normal[0] * self.loc[0] -
				hm_normal[2] * self.loc[2]) /
				hm_normal[1];

		// Apply accelerations

		// Acceleration such that we reach max_speed in five frames
		let accel = self.decel + (self.max_speed / 5.0);
		let jump_accel = self.gravity + (self.max_jump / 5.0);

		if movement.forward {
			self.vel[0] += dir[0] * accel;
			self.vel[2] += dir[2] * accel;
		}
		if movement.backward {
			self.vel[0] -= dir[0] * accel;
			self.vel[2] -= dir[2] * accel;
		}
		if movement.left {
			self.vel[0] -= dir[2] * accel;
			self.vel[2] += dir[0] * accel;
		}
		if movement.right {
			self.vel[0] += dir[2] * accel;
			self.vel[2] -= dir[0] * accel;
		}
		if movement.jumping {
			if self.loc[1] <= height {
				movement.can_jump = 5;
				self.vel[1] += jump_accel;
			} else if movement.can_jump > 0 {
				movement.can_jump -= 1;
				self.vel[1] += jump_accel;
			}
		}

		// Apply decelerations

		let char_speed = f32::hypot(self.vel[0], self.vel[2]);
		let multiplier = if char_speed - self.decel > self.max_speed {
			self.max_speed / char_speed } else {
			f32::max(0.0, (char_speed - self.decel) / char_speed)};
		self.vel[0] *= multiplier;
		self.vel[2] *= multiplier;

		// Gravity:
		self.vel[1] -= self.gravity;

		// Update locations
		self.loc[0] += self.vel[0];
		self.loc[1] += self.vel[1];
		self.loc[2] += self.vel[2];


		// Collision with ground
		if self.loc[1] <= height {
			self.loc[1] = height;
			self.vel[1] = 0.0;
		}
	}

	/// Get the location of this character.
	pub fn loc(&self) -> &Vec3<f32> {
		&self.loc
	}
}
