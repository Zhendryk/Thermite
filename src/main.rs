use gl::{self, types::*};
use glfw::{self, Action, Context, Key};
use std::{os::raw::c_void, path::Path, rc::Rc, sync::mpsc::Receiver};

pub mod opengl;
use opengl::{shaders, vertex_buffer::VertexBuffer};

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

    let vao = unsafe {
        // Vertex input (type annotate to f32 as f64 is the default)
        let vertices: [f32; 9] = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

        // Create our Vertex Array Object and Vertex Buffer Object
        let (mut vbo, mut vao) = (0, 0);
        gl.GenBuffers(1, &mut vbo);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

        // Vao
        gl.GenVertexArrays(1, &mut vao);
        gl.BindVertexArray(vao);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

        // Specify a vertex attribute array for the position data in our vbo we specified & enable it
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<GLfloat>() as GLsizei,
            std::ptr::null(),
        );

        // We can unbind the vbo now that it is bound to the vao
        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        // vbo.unbind(&gl);

        // Unbind vao for now to avoid it being mutated by accident (rarely happens)
        gl.BindVertexArray(0);

        // Uncomment this to draw wireframe polygons
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        vao
    };

    while !window.should_close() {
        process_events(&gl, &mut window, &events);

        unsafe {
            gl.ClearColor(0.2, 0.3, 0.3, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            shader_program.use_program();
            // gl.UseProgram(shader_program);
            gl.BindVertexArray(vao);
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
