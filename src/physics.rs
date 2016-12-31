use MovementState;

pub fn do_char_movement(character: &mut CharacterState,
		dir: &[f32; 3],
		movement: &mut MovementState) {

	// Apply accelerations

	// Acceleration such that we reach max_speed in five frames
	let accel = character.decel + (character.max_speed / 5.0);

	if movement.forward {
		character.vel[0] += dir[0] * accel;
		character.vel[2] += dir[2] * accel;
	}
	if movement.backward {
		character.vel[0] -= dir[0] * accel;
		character.vel[2] -= dir[2] * accel;
	}
	if movement.left {
		character.vel[0] -= dir[2] * accel;
		character.vel[2] += dir[0] * accel;
	}
	if movement.right {
		character.vel[0] += dir[2] * accel;
		character.vel[2] -= dir[0] * accel;
	}
	if movement.jumping {
		if character.loc[1] <= 0.0 {
			movement.can_jump = 5;
			character.vel[1] += 0.05;
		} else if movement.can_jump > 0 {
			movement.can_jump -= 1;
			character.vel[1] += 0.05;
		}
	}

	// Apply decelerations

	let char_speed = f32::sqrt(
		character.vel[0] * character.vel[0] +
		character.vel[2] * character.vel[2]);
	let multiplier = if char_speed - character.decel > character.max_speed {
		character.max_speed / char_speed } else {
		f32::max(0.0, (char_speed - character.decel) / char_speed)};
	character.vel[0] *= multiplier;
	character.vel[2] *= multiplier;

	// Gravity:
	character.vel[1] -= 0.02;

	// Update locations
	character.loc[0] += character.vel[0];
	character.loc[1] += character.vel[1];
	character.loc[2] += character.vel[2];

	// Collision with ground
	if character.loc[1] <= 0.0 {
		character.loc[1] = 0.0;
		character.vel[1] = 0.0;
	}
}

#[derive(Debug)]
pub struct CharacterState {
	loc: [f32; 3],
	vel: [f32; 3],
	max_speed: f32,
	decel: f32,
	max_jump: f32,
	gravity: f32
}
impl CharacterState {
	pub fn new(loc: [f32; 3],
			vel: [f32; 3],
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

	pub fn loc(&self) -> &[f32; 3] {
		&self.loc
	}

}
