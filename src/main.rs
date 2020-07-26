// Conditionally compile the proper gfx backend as thermite_gfx_backend
#[cfg(feature = "dx12")]
use gfx_backend_dx12 as thermite_gfx_backend;
// #[cfg(feature = "opengl")]
// use gfx_backend_gl as thermite_gfx_backend;
#[cfg(feature = "metal")]
use gfx_backend_metal as thermite_gfx_backend;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as thermite_gfx_backend;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::{
    self,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() {}

// fn main() {
//     simple_logger::init().unwrap();
//     let mut window = Window::default();
//     // move forces the closure to take ownership of the captured data in its environment (e.g. window)
//     window.event_loop().run(move |event, _, control_flow| {
//         *control_flow = ControlFlow::Wait;
//         match event {
//             Event::WindowEvent {
//                 event: WindowEvent::CloseRequested,
//                 window_id,
//             } if window_id == window.id() => *control_flow = ControlFlow::Exit,
//             _ => (),
//         }
//     });
// }
