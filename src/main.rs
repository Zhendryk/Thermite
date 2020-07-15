use gl;
use glfw::{self, WindowHint, WindowMode};
use std::path::Path;

pub mod opengl;
use opengl::{
    buffer_layout::{BufferComponent, BufferComponentType, BufferLayout},
    index_buffer::IndexBuffer,
    shaders,
    vertex_array::VertexArray,
    vertex_buffer::VertexBuffer,
    window::GLFWWindow,
};

pub mod resources;
use resources::Resource;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

fn main() {
    let hints = vec![
        WindowHint::ContextVersion(3, 3),
        WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core),
        WindowHint::OpenGlForwardCompat(true),
    ];
    let mut window = GLFWWindow::new(
        WIDTH,
        HEIGHT,
        "Thermite Engine v0.1.0",
        WindowMode::Windowed,
        glfw::FAIL_ON_ERRORS,
        Option::from(hints),
    )
    .expect("Failed to create GLFW window");

    window.make_context_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    let gl = window.load_opengl_fn_ptrs();

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
        window.process_events(&gl);

        unsafe {
            gl.ClearColor(0.2, 0.3, 0.3, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            shader_program.use_program();
            vao.bind();
            gl.DrawArrays(gl::TRIANGLES, 0, 3);
            // gl.DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
        }

        window.poll_events();
        window.swap_buffers();
    }
}
