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
	loop {
		let mut target = display.draw();
		target.clear_color(0.5, 0.5, 1.0, 1.0);
		// More drawing commands here
		target.finish().unwrap();

		// Handle events
		for ev in display.poll_events() {
			match ev {
				Event::Closed => {
					println!("Got Closed event, exiting...");
					return; },
				x => println!("Got event: {:?}", x)
			}
		}
	}
}
