extern crate glium;
use glium::glutin::Window;

pub struct Camera {
	pub loc_x: f32,
	pub loc_y: f32,
	pub loc_z: f32,
	pub dir_x: f32,
	pub dir_y: f32,
	pub dir_z: f32
}

pub fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {

    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s[2] - f[2] * s[1],
             f[2] * s[0] - f[0] * s[2],
             f[0] * s[1] - f[1] * s[0]];

    let p = [-position[0] * s[0] - position[1] * s[1] - position[2] * s[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s[0], u[0], f[0], 0.0],
        [s[1], u[1], f[1], 0.0],
        [s[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]

}

pub fn perspective_matrix(width: u32, height: u32, fov: f32) -> [[f32; 4]; 4] {
	let aspect_ratio = height as f32 / width as f32;

	let zfar = 1024.0;
	let znear = 0.1;

	let f = 1.0 / (fov / 2.0).tan();

	[
		[f * aspect_ratio, 0.0, 0.0,                            0.0],
		[0.0             , f,   0.0,                            0.0],
		[0.0             , 0.0, (zfar+znear)/(zfar-znear),      1.0],
		[0.0             , 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0],
	]
}

pub fn handle_mouse_move(window: &Window, camera: &mut Camera, x: i32, y: i32) -> () {
	let (uw, uh) = window.get_inner_size().unwrap();
	let w = uw as i32;
	let h = uh as i32;
	window.set_cursor_position(w/2, h/2).unwrap();
	let dx = w/2 - x;
	let dy = h/2 - y;
	if dx.abs() > 200 || dy.abs() > 200 {
		info!("Skipping camera move due to large delta: {}, {}", dx, dy);
		return;
	}

	// Turn dx into a rotation on the xz plane
	let dh = dx as f32 * 0.005;
	let new_dir_x = camera.dir_x * dh.cos() - camera.dir_z * dh.sin();
	let new_dir_z = camera.dir_x * dh.sin() + camera.dir_z * dh.cos();
	camera.dir_x = new_dir_x;
	camera.dir_z = new_dir_z;

	// Turn dy into a rotation on the xy plane
	// (not really the xy plane; it's the plane determined by xz and [0,1,0])
	// Clamp dir_y + dy
	// (the camera will flip if you cross zenith or nadir, which is super confusing)
	//FIXME: This more-or-less works, but is probably^Wdefinitely wrong.
	camera.dir_y += dy as f32 * 0.005;

}

