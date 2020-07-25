// Conditionally compile the proper gfx backend as gfx_backend
#[cfg(feature = "dx12")]
use gfx_backend_dx12 as gfx_backend;
#[cfg(feature = "opengl")]
use gfx_backend_gl as gfx_backend;
#[cfg(feature = "metal")]
use gfx_backend_metal as gfx_backend;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as gfx_backend;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::{
    self,
    dpi::LogicalSize,
    error::OsError,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowId},
};

#[derive(Debug)]
pub struct Window {
    handle: winit::window::Window,
    title: String,
    size: LogicalSize<u16>,
    event_loop: Option<EventLoop<()>>,
    should_close: bool,
}

impl Window {
    /// Constructs a new `Window` with the given `title` and `size`.
    ///
    /// It's possible for the window creation to fail (`OsError`), but this is unlikely.
    pub fn new<T: Into<String>>(title: T, size: LogicalSize<u16>) -> Result<Self, OsError> {
        let event_loop = EventLoop::new();
        let title_str = title.into();
        let window = WindowBuilder::new()
            .with_title(title_str.clone())
            .with_inner_size(size)
            .build(&event_loop)?;
        Ok(Self {
            handle: window,
            title: title_str,
            size: size,
            event_loop: Option::from(event_loop),
            should_close: false,
        })
    }

    /// Returns this `Window`'s unique identifier
    pub fn id(&self) -> winit::window::WindowId {
        self.handle.id()
    }

    /// Returns a reference to the title of this `Window`
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns a reference to the dimensions of this `Window`
    pub fn size(&self) -> &winit::dpi::LogicalSize<u16> {
        &self.size
    }

    /// Returns the `EventLoop` associated with this `Window`
    pub fn event_loop(&mut self) -> winit::event_loop::EventLoop<()> {
        self.event_loop
            .take()
            .expect("Could not retreive the window's event loop!")
    }

    /// Returns a reference to whether or not this `Window` has been signaled to close
    pub fn should_close(&self) -> &bool {
        &self.should_close
    }
}

impl Default for Window {
    /// Makes an 800x600 window with the `Thermite Engine` as the title.
    ///
    /// ### Panics
    /// If a `OsError` occurs.
    fn default() -> Self {
        Self::new(
            "Thermite Engine",
            LogicalSize {
                width: 800,
                height: 600,
            },
        )
        .expect("Could not create a window!")
    }
}

// fn main() {
//     simple_logger::init().unwrap();
//     let mut window = Window::default();
//     // move forces the closure to take ownership of the captured data in it's environment (e.g. window)
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
