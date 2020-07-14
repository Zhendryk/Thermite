use gl::{self, types::*};
use glfw::{self, Action, Context, Key};
use std::{os::raw::c_void, path::Path, rc::Rc, sync::mpsc::Receiver};

pub mod opengl;
use opengl::{
    buffer_layout::{BufferComponent, BufferComponentType, BufferLayout},
    index_buffer::IndexBuffer,
    shaders,
    vertex_array::VertexArray,
    vertex_buffer::VertexBuffer,
};

pub mod resources;
use resources::Resource;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

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
    let shader_program = shaders::ShaderProgram::new(&shaders, "colored", &gl).unwrap();

    // Create our Vertex Array Object and Vertex Buffer Object
    // let vertices: [f32; 12] = [
    //     0.5, 0.5, 0.0, // Bottom right
    //     0.5, -0.5, 0.0, // Top right
    //     -0.5, -0.5, 0.0, // Top left
    //     -0.5, 0.5, 0.0, // Bottom left
    // ];
    // let indices: [u32; 6] = [0, 1, 3, 1, 2, 3];
    let vertices: [f32; 18] = [
        // positions   //colors
        0.5, -0.5, 0.0, 1.0, 0.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, // bottom left
        0.0, 0.5, 0.0, 0.0, 0.0, 1.0, // top
    ];
    let positions = BufferComponent::new(
        String::from("positions"),
        BufferComponentType::Float3,
        false,
    );
    let colors = BufferComponent::new(String::from("colors"), BufferComponentType::Float3, true);
    let layout = BufferLayout::new(&mut [positions, colors]);

    let vbo = VertexBuffer::new(&gl, layout);
    vbo.bind();
    vbo.buffer_data(&vertices, gl::STATIC_DRAW);
    vbo.unbind();
    // let ibo = IndexBuffer::new(&gl);
    // ibo.bind();
    // ibo.buffer_data(&indices, gl::STATIC_DRAW);
    // ibo.unbind();

    let mut vao = VertexArray::new(&gl);
    vao.add_vertex_buffer(vbo);
    // vao.set_index_buffer(ibo);

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
            // gl.DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
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
