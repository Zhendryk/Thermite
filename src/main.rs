#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use simple_logger;

use thermite_gfx::{gfx_hal::window::Extent2D, hal_state::HALState};
use thermite_ui::window;

fn main() {
    simple_logger::init().expect("Couldn't create simple logger");
    let mut should_configure_swapchain = true;
    let mut window = window::Window::default();
    let mut hal_state = HALState::new(window.handle()).expect("Couldn't create HALState");
    let mut surface_extent = Extent2D {
        width: window.physical_size().width,
        height: window.physical_size().height,
    };
    window.event_loop().run(move |event, _, control_flow| {
        use thermite_ui::winit::{
            event::{DeviceEvent, Event, VirtualKeyCode, WindowEvent},
            event_loop::ControlFlow,
        };
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::Key(key) => {
                    if key.virtual_keycode == Some(VirtualKeyCode::Escape) {
                        *control_flow = ControlFlow::Exit
                    }
                }
                _ => (),
            },
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
                // NOTE: perform rendering here
                unsafe {
                    hal_state.resources.reset_command_pool(1_000_000_000);
                };
                if should_configure_swapchain {
                    surface_extent = hal_state.resources.recreate_swapchain(surface_extent);
                    should_configure_swapchain = false;
                }
                let surface_image = unsafe {
                    hal_state
                        .resources
                        .acquire_image(1_000_000_000)
                        .map_err(|e| {
                            should_configure_swapchain = e;
                            return;
                        })
                        .expect("Couldn't acquire surface image")
                };
                let framebuffer = unsafe {
                    hal_state
                        .resources
                        .create_framebuffer(&surface_image, surface_extent)
                        .expect("Couldn't create framebuffer!")
                };
                let viewport = hal_state.resources.viewport(surface_extent);
                unsafe {
                    hal_state
                        .resources
                        .record_cmds_for_submission(&framebuffer, &viewport);
                    should_configure_swapchain |=
                        hal_state.resources.submit_cmds(surface_image, framebuffer);
                };
            }
            _ => (),
        }
    });
}
