use gl;
use glfw::{self, Action, Context, ErrorCallback, InitError, Key, WindowEvent, WindowHint};

/// An application window created by GLFW
pub struct GLFWWindow {
    handle: glfw::Window,
    glfw: glfw::Glfw,
    width: u32,
    height: u32,
    title: String,
    event_receiver: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
}

impl GLFWWindow {
    /// Creates a new `GLFWWindow`
    ///
    /// ### Parameters
    ///
    /// - `width`: The width (pixels) of the window
    /// - `height`: The height (pixels) of the window
    /// - `title`: The title text of the window
    /// - `mode`: The mode of the window (Fullscreen|Windowed)
    /// - `error_callback`: The `glfw::ErrorCallback` to use in the event of an error during initialization
    /// - `hints`: Setup parameters for the `GLFWWindow`
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: A newly initialized `GLFWWindow`
    /// - `Err`: A `glfw::InitError` describing what went wrong during window creation
    pub fn new(
        width: u32,
        height: u32,
        title: &str,
        mode: glfw::WindowMode,
        error_callback: Option<ErrorCallback<()>>,
        hints: Option<Vec<WindowHint>>,
    ) -> Result<GLFWWindow, InitError> {
        let mut glfw = glfw::init(error_callback)?;
        for hint in hints.unwrap_or_default() {
            glfw.window_hint(hint);
        }
        let (window, event_receiver) = glfw
            .create_window(width, height, title, mode)
            .expect("Failed to create GLFW window.");

        Ok(GLFWWindow {
            handle: window,
            glfw: glfw,
            width: width,
            height: height,
            title: title.to_owned(),
            event_receiver: event_receiver,
        })
    }

    /// Returns an immutable reference to the width (pixels) of this `GLFWWindow`
    pub fn width(&self) -> &u32 {
        &self.width
    }

    /// Returns an immutable reference to the height (pixels) of this `GLFWWindow`
    pub fn height(&self) -> &u32 {
        &self.height
    }

    /// Returns an immutable reference to the title text of this `GLFWWindow`
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Wrapper for `glfwMakeContextCurrent`
    pub fn make_context_current(&mut self) {
        self.handle.make_current();
    }

    /// Wrapper for `glfwSetKeyCallback`
    pub fn set_key_polling(&mut self, should_poll: bool) {
        self.handle.set_key_polling(should_poll);
    }

    /// Wrapper for `glfwSetFramebufferSizeCallback`
    pub fn set_framebuffer_size_polling(&mut self, should_poll: bool) {
        self.handle.set_framebuffer_size_polling(should_poll);
    }

    /// Swaps the front and back buffers of the window. If the swap interval is greater than zero, the GPU driver waits the specified number of screen updates before swapping the buffers.
    pub fn swap_buffers(&mut self) {
        self.handle.swap_buffers();
    }

    /// Immediate process received events
    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

    /// Wrapper for `glfwWindowShouldClose`
    pub fn should_close(&self) -> bool {
        self.handle.should_close()
    }

    /// Process/handle all pending events in this `GLFWWindow`'s event receiver
    pub fn process_events(&mut self, gl: &gl::Gl) {
        for (_, event) in glfw::flush_messages(&self.event_receiver) {
            match event {
                WindowEvent::FramebufferSize(width, height) => unsafe {
                    gl.Viewport(0, 0, width, height)
                },
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.handle.set_should_close(true)
                }
                _ => {}
            }
        }
    }

    /// Load OpenGL function pointers and return it as a reference counted pointer object
    pub fn load_opengl_fn_ptrs(&mut self) -> std::rc::Rc<gl::Gl> {
        std::rc::Rc::new(gl::Gl::load_with(|symbol| {
            self.handle.get_proc_address(symbol) as *const std::os::raw::c_void
        }))
    }
}
