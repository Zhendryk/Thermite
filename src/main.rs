use gl::{self, types::*};
use glfw::{self, Action, Context, Key};
use std::{path::Path, rc::Rc, sync::mpsc::Receiver};

pub mod opengl;
use opengl::{shaders, vertex_array::VertexArray, vertex_buffer::VertexBuffer};

pub mod resources;
use resources::Resource;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn main() {
    // Initialize GLFW
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(
            WIDTH,
            HEIGHT,
            "Thermite Engine v0.1.0",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // load all OpenGL function pointers
    let gl = Rc::new(gl::Gl::load_with(|symbol| {
        window.get_proc_address(symbol) as *const std::os::raw::c_void
    }));

    // Grab our shader resource and create a shader program with it
    let shaders = Resource::new(Path::new("assets/shaders")).unwrap();
    let shader_program = shaders::ShaderProgram::new(&shaders, "basic", &gl).unwrap();

    // Create our Vertex Array Object and Vertex Buffer Object
    let vertices: [f32; 9] = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];
    let vbo = VertexBuffer::new(&gl);
    vbo.bind();
    vbo.buffer_data(&vertices, gl::STATIC_DRAW);
    vbo.unbind();

    let vao = VertexArray::new(&gl);
    vao.bind();
    vbo.bind();

    unsafe {
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<GLfloat>() as GLsizei,
            std::ptr::null(),
        );
    }

    vbo.unbind();
    vao.unbind();

    // Uncomment this to draw wireframe polygons
    // unsafe {
    //     gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    // }

    while !window.should_close() {
        process_events(&gl, &mut window, &events);

        unsafe {
            gl.ClearColor(0.2, 0.3, 0.3, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            shader_program.use_program();
            vao.bind();
            gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }

        glfw.poll_events();
        window.swap_buffers();
    }
}

fn process_events(
    gl: &gl::Gl,
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,
) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                gl.Viewport(0, 0, width, height)
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
