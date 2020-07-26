#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use simple_logger;

use thermite_ui::window;
use thermite_ui::winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() {
    simple_logger::init().expect("Couldn't create simple logger");
    let mut window = window::Window::default();
    // move forces the closure to take ownership of the captured data in its environment (e.g. window)
    window.event_loop().run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
