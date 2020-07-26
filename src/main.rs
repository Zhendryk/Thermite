#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use simple_logger;

use thermite_gfx::gfx_hal::window::Extent2D;
use thermite_ui::window;

fn main() {
    simple_logger::init().expect("Couldn't create simple logger");
    let mut should_configure_swapchain = true;
    let mut window = window::Window::default();
    let mut surface_extent = Extent2D {
        width: window.physical_size().width,
        height: window.physical_size().height,
    };
    window.event_loop().run(move |event, _, control_flow| {
        use thermite_ui::winit::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
        };
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(dims) => {
                    surface_extent = Extent2D {
                        width: dims.width,
                        height: dims.height,
                    };
                    should_configure_swapchain = true;
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    surface_extent = Extent2D {
                        width: new_inner_size.width,
                        height: new_inner_size.height,
                    };
                    should_configure_swapchain = true;
                }
                _ => (),
            },
            Event::MainEventsCleared => window.handle().request_redraw(),
            Event::RedrawRequested(_) => {
                // perform rendering here
            }
            _ => (),
        }
    });
}
