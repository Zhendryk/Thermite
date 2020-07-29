use winit::{
    self,
    dpi::{LogicalSize, PhysicalSize},
    error::OsError,
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[derive(Debug)]
pub struct Window {
    handle: winit::window::Window,
    title: String,
    logical_size: LogicalSize<u32>,
    physical_size: PhysicalSize<u32>,
    dpi: f64,
    event_loop: Option<EventLoop<()>>,
    should_close: bool,
}

// TODO: Try and see if we can encapsulate user input related to the window into
//       some function which accepts a map of input->callback or something..., that
//       way we don't need a huge input loop in our main.rs.
// TODO (cont.): See if ^^ this can also apply to the event loop
impl Window {
    /// Constructs a new `Window` with the given `title` and `size`.
    ///
    /// It's possible for the window creation to fail (`OsError`), but this is unlikely.
    pub fn new<T: Into<String>>(title: T, size: [u32; 2]) -> Result<Self, OsError> {
        let event_loop = EventLoop::new();
        let title_str = title.into();
        let (logical_size, physical_size, dpi) = {
            let dpi = event_loop.primary_monitor().scale_factor();
            let logical: LogicalSize<u32> = size.into();
            let physical: PhysicalSize<u32> = logical.to_physical(dpi.clone());
            (logical, physical, dpi)
        };
        let window = WindowBuilder::new()
            .with_title(title_str.clone())
            .with_inner_size(logical_size.clone())
            .build(&event_loop)?;
        Ok(Self {
            handle: window,
            title: title_str,
            logical_size: logical_size,
            physical_size: physical_size,
            dpi: dpi,
            event_loop: Option::from(event_loop),
            should_close: false,
        })
    }

    /// Returns this `Window`'s unique identifier
    pub fn id(&self) -> winit::window::WindowId {
        self.handle.id()
    }

    /// Returns a reference to the winit handle for this `Window`
    pub fn handle(&self) -> &winit::window::Window {
        &self.handle
    }

    /// Returns a reference to the title of this `Window`
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns a reference to the logical (physical pixels scaled to dpi) dimensions of this `Window`
    pub fn logical_size(&self) -> &winit::dpi::LogicalSize<u32> {
        &self.logical_size
    }

    /// Returns a reference to the physical (actual number of pixels) dimensions of this `Window`
    pub fn physical_size(&self) -> &winit::dpi::PhysicalSize<u32> {
        &self.physical_size
    }

    /// Returns a reference to the dpi (dots per inch) of this `Window`
    pub fn dpi(&self) -> &f64 {
        &self.dpi
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
            format!(
                "Thermite Engine - thermite_ui v{}",
                env!("CARGO_PKG_VERSION")
            ),
            [800, 600],
        )
        .expect("Could not create a window!")
    }
}
