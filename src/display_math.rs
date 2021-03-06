//! Vector math for display transformations.

use errors::*;
use glium::glutin::Window;
use linear_algebra::{Mat4, Vec3};

/// Representation of a camera: location and direction.
#[derive(Debug)]
pub struct Camera {
	/// Location of this camera.
	pub loc: Vec3<f32>,
	/// Direction of this camera.
	pub dir: Vec3<f32>,
}

/// Compute a view transformation matrix based on the given parameters.
///
/// This transformation is mostly standard; see [OpenGL
/// `gluLookAt`](https://www.opengl.org/sdk/docs/man2/xhtml/gluLookAt.xml) for
/// a detailed description of what it does and how it works.
pub fn view_matrix(position: Vec3<f32>, direction: Vec3<f32>, up: Vec3<f32>) -> Mat4<f32> {

	let f = direction.normalize();

	let s = up.cross(f).normalize();

	let u = f.cross(s);

	let p = [-position[0] * s[0] - position[1] * s[1] - position[2] * s[2],
	         -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
	         -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

	Mat4::from([
		[s[0], u[0], f[0], 0.0],
		[s[1], u[1], f[1], 0.0],
		[s[2], u[2], f[2], 0.0],
		[p[0], p[1], p[2], 1.0],
	])

}

/// Compute a perspective matrix based on the given parameters.
///
/// This transformation is mostly standard; see [OpenGL
/// `gluPerspective`](https://www.opengl.org/sdk/docs/man2/xhtml/gluPerspective.xml)
/// for a detailed description of what it does and how it works.
pub fn perspective_matrix(width: u32, height: u32, fov: f32) -> Mat4<f32> {
	let aspect_ratio = height as f32 / width as f32;

	let zfar = 1048576.0;
	let znear = 0.1;

	let f = 1.0 / (fov / 2.0).tan();

	Mat4::from([
		[f * aspect_ratio, 0.0, 0.0,                            0.0],
		[0.0             , f,   0.0,                            0.0],
		[0.0             , 0.0, (zfar+znear)/(zfar-znear),      1.0],
		[0.0             , 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],
	])
}

/// Handle mouse movement.
///
/// This translates mouse x/y movement into a change of the direction of the
/// given `Camera`, and keeps the mouse captured within the window.
///
/// Very large mouse movements (typically due to gaining focus with the cursor
/// in a different location than last seen) will be ignored.
///
/// TODO: The mouse capture and focus management should be handled elsewhere.
pub fn handle_mouse_move(window: &Window, camera: &mut Camera, x: f64, y: f64) -> Result<()> {

	// Capture the mouse
	let (w, h): (u32, u32) = try!{
		window.get_inner_size()
			.map(|s| s.into())
			.ok_or(Error::from("Could not get window size"))
	};
	try!{ window.set_cursor_position((w as i32/2, h as i32/2).into())
			.map_err(|_| { Error::from("Could not set cursor position") } ) };

	if x.abs() > 200.0 || y.abs() > 200.0 {
		info!("Skipping camera move due to large delta: {}, {}", x, y);
		return Ok(());
	}

	// Turn dx into a rotation on the xz plane
	let dh = x as f32 * -0.005;
	camera.dir[0] = camera.dir[0] * dh.cos() - camera.dir[2] * dh.sin();
	camera.dir[2] = camera.dir[0] * dh.sin() + camera.dir[2] * dh.cos();
	// Accumulated error will lead to movement glitches if we don't renormalize this.
	let xz_norm = f32::hypot(camera.dir[0], camera.dir[2]);
	camera.dir[0] /= xz_norm;
	camera.dir[2] /= xz_norm;

	// Turn dy into a rotation on the xy plane
	// (not really the xy plane; it's the plane determined by x and [0,1,0])
	// Clamp dir_y + dy
	// (otherwise the camera will flip if you cross zenith or nadir, which is super confusing)
	//FIXME: This more-or-less works, but is probably^Wdefinitely wrong.
	camera.dir[1] += y as f32 * -0.005;

	Ok(())
}

