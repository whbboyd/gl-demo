extern crate glium;
use glium::DisplayBuild;
use glium::Surface;
use glium::glutin::Event;

fn main() {
	println!("Starting demo...");

	println!("Initializing window...");
	let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

	// Main program loop
	println!("Starting program loop...");
	'main: loop {
		let mut target = display.draw();
		target.clear_color(0.5, 0.5, 1.0, 1.0);
		// More drawing commands here
		target.finish().unwrap();

		// Handle events
		for ev in display.poll_events() {
			println!("Got event: {:?}", ev);
			match ev {
				Event::Closed => break 'main,
				_ => ()
			}
		}

		// Wait for end of frame
	}

	println!("Program loop ended, exiting...");
}
