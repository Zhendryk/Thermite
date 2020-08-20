use winit::{
    self,
    dpi::LogicalSize,
    error::OsError,
    event_loop::EventLoop,
    window::{Window as WinitWindow, WindowAttributes, WindowBuilder},
};

#[derive(Debug)]
pub struct Window<L: 'static> {
    handle: WinitWindow, // ! Attribute altering functions accessed through here
    event_loop: Option<EventLoop<L>>,
}

impl<L: 'static> Window<L> {
    /// Constructs a new `Window` with the given `title` and `size`.
    ///
    /// It's possible for the window creation to fail (`OsError`), but this is unlikely.
    pub fn new<T>(title: T, size: [u32; 2]) -> Result<Self, OsError>
    where
        T: Into<String>,
    {
        let event_loop = EventLoop::<L>::with_user_event();
        let logical_pixel_size: LogicalSize<u32> = size.into();
        Ok(Self {
            handle: WindowBuilder::new()
                .with_title(title)
                .with_inner_size(logical_pixel_size.clone())
                .build(&event_loop)?,
            event_loop: Option::from(event_loop),
        })
    }

    /// Creates a `Window` using the given `WindowAttributes`
    pub fn from_attributes(attributes: WindowAttributes) -> Result<Self, OsError> {
        let event_loop = EventLoop::<L>::with_user_event();
        let mut builder = WindowBuilder::new();
        builder.window = attributes;
        Ok(Self {
            handle: builder.build(&event_loop)?,
            event_loop: Option::from(event_loop),
        })
    }

    /// Returns a reference to the winit handle for this `Window`
    pub fn handle(&self) -> &WinitWindow {
        &self.handle
    }

    /// Moves the `EventLoop` associated with this `Window` out of it for usage.
    ///
    /// **NOTE:** Can only be done once!
    pub fn event_loop(&mut self) -> EventLoop<L> {
        self.event_loop
            .take()
            .expect("Cannot take more than one event loop from the window!")
    }
}

impl<L: 'static> Default for Window<L> {
    /// Makes an 800x600 window with the `Thermite Engine` as the title.
    ///
    /// ### Panics
    /// If a `OsError` occurs.
    fn default() -> Self {
        Self::new(
            format!("Thermite Engine v{}", env!("CARGO_PKG_VERSION")),
            [800, 600],
        )
        .expect("Could not create Thermite Engine window!")
    }
}
