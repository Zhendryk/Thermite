use crate::opengl::camera::{Camera, CameraMovementDirection};
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

    /// Returns a reference to the width (pixels) of this `GLFWWindow`
    pub fn width(&self) -> &u32 {
        &self.width
    }

    /// Returns a reference to the height (pixels) of this `GLFWWindow`
    pub fn height(&self) -> &u32 {
        &self.height
    }

    /// Returns a reference to the title text of this `GLFWWindow`
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Wrapper for `glfwMakeContextCurrent`
    pub fn make_context_current(&mut self) {
        self.handle.make_current()
    }

    /// Wrapper for `glfwSetInputMode` called with `CURSOR`
    pub fn set_cursor_mode(&mut self, cursor_mode: glfw::CursorMode) {
        self.handle.set_cursor_mode(cursor_mode)
    }

    /// Wrapper for `glfwSetKeyCallback`
    pub fn set_key_polling(&mut self, should_poll: bool) {
        self.handle.set_key_polling(should_poll)
    }

    /// Wrapper for `glfwSetScrollCallback`
    pub fn set_scroll_polling(&mut self, should_poll: bool) {
        self.handle.set_scroll_polling(should_poll)
    }

    /// Wrapper for `glfwSetCursorPosCallback`
    pub fn set_cursor_pos_polling(&mut self, should_poll: bool) {
        self.handle.set_cursor_pos_polling(should_poll)
    }

    /// Wrapper for `glfwSetFramebufferSizeCallback`
    pub fn set_framebuffer_size_polling(&mut self, should_poll: bool) {
        self.handle.set_framebuffer_size_polling(should_poll)
    }

    /// Swaps the front and back buffers of the window. If the swap interval is greater than zero, the GPU driver waits the specified number of screen updates before swapping the buffers.
    pub fn swap_buffers(&mut self) {
        self.handle.swap_buffers()
    }

    /// Immediate process received events
    pub fn poll_events(&mut self) {
        self.glfw.poll_events()
    }

    /// Wrapper for `glfwWindowShouldClose`
    pub fn should_close(&self) -> bool {
        self.handle.should_close()
    }

    /// Get the current value of the GLFW timer
    pub fn get_time(&self) -> f64 {
        self.glfw.get_time()
    }

    /// Process/handle all pending events in this `GLFWWindow`'s event receiver
    pub fn process_events(
        &mut self,
        gl: &gl::Gl,
        delta_time: &f32,
        last_x: &mut f64,
        last_y: &mut f64,
        first_mouse: &mut bool,
        camera: &mut Camera,
    ) {
        for (_, event) in glfw::flush_messages(&self.event_receiver) {
            match event {
                WindowEvent::FramebufferSize(width, height) => unsafe {
                    gl.Viewport(0, 0, width, height)
                },
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.handle.set_should_close(true)
                }
                WindowEvent::Key(Key::W, _, Action::Press, _) => {
                    camera.process_keyboard(CameraMovementDirection::FORWARD, delta_time)
                }
                WindowEvent::Key(Key::W, _, Action::Repeat, _) => {
                    camera.process_keyboard(CameraMovementDirection::FORWARD, delta_time)
                }
                WindowEvent::Key(Key::S, _, Action::Press, _) => {
                    camera.process_keyboard(CameraMovementDirection::BACKWARD, delta_time)
                }
                WindowEvent::Key(Key::S, _, Action::Repeat, _) => {
                    camera.process_keyboard(CameraMovementDirection::BACKWARD, delta_time)
                }
                WindowEvent::Key(Key::A, _, Action::Press, _) => {
                    camera.process_keyboard(CameraMovementDirection::LEFT, delta_time)
                }
                WindowEvent::Key(Key::A, _, Action::Repeat, _) => {
                    camera.process_keyboard(CameraMovementDirection::LEFT, delta_time)
                }
                WindowEvent::Key(Key::D, _, Action::Press, _) => {
                    camera.process_keyboard(CameraMovementDirection::RIGHT, delta_time)
                }
                WindowEvent::Key(Key::D, _, Action::Repeat, _) => {
                    camera.process_keyboard(CameraMovementDirection::RIGHT, delta_time)
                }
                WindowEvent::Scroll(_, y_offset) => camera.process_mouse_scroll(y_offset as f32),
                WindowEvent::CursorPos(x_pos, y_pos) => {
                    if *first_mouse {
                        *last_x = x_pos;
                        *last_y = y_pos;
                        *first_mouse = false;
                    }
                    let x_offset = x_pos - *last_x;
                    let y_offset = *last_y - y_pos;
                    *last_x = x_pos;
                    *last_y = y_pos;
                    camera.process_mouse_move(x_offset as f32, y_offset as f32, true)
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
