use gl;
use glfw::{self, WindowHint, WindowMode};
use std::path::{Path, PathBuf};

pub mod opengl;
use opengl::{
    buffer_layout::{BufferComponent, BufferComponentType, BufferLayout},
    index_buffer::IndexBuffer,
    shaders,
    texture::Texture,
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
    let shaders = Resource::new(Path::new("assets/shaders"))
        .expect("Could not create resource from assets/shaders");
    let shader_program = shaders::ShaderProgram::new(&shaders, "textured", &gl).unwrap();

    let vertices: [f32; 32] = [
        // positions          // colors           // texture coords
        0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
        0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
        -0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // top left
    ];
    let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
    // TODO: Load texture from `Resource`, incorporate into Texture::new
    // let textures = Resource::new(Path::new("assets/textures"))
    //     .expect("Could not create resource from assets/textures");
    let texture = Texture::new(
        &PathBuf::from("assets/textures/wall.jpg"),
        gl::TEXTURE_2D,
        gl::RGB,
        gl::RGB,
        &gl,
    )
    .expect("Could not load wall texture");
    texture.bind();
    texture.set_texture_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture.set_texture_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture.set_texture_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR);
    texture.set_texture_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    texture.generate_with_mipmap();
    let positions = BufferComponent::new(
        String::from("positions"),
        BufferComponentType::Float3,
        false,
    );
    let colors = BufferComponent::new(String::from("colors"), BufferComponentType::Float3, false);
    let texture_coords = BufferComponent::new(
        String::from("tex-coords"),
        BufferComponentType::Float2,
        true,
    );
    let layout = BufferLayout::new(&mut [positions, colors, texture_coords]);
    // Create our vao, vbo and ibo
    let mut vao = VertexArray::new(&gl);
    let vbo = VertexBuffer::new(&gl, layout);
    vbo.bind();
    vbo.buffer_data(&vertices, gl::STATIC_DRAW);
    vao.add_vertex_buffer(vbo);
    let ibo = IndexBuffer::new(&gl);
    ibo.bind();
    ibo.buffer_data(&indices, gl::STATIC_DRAW);
    vao.set_index_buffer(ibo);
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
            texture.bind();
            shader_program.use_program();
            vao.bind();
            // gl.DrawArrays(gl::TRIANGLES, 0, 3); // For non-ibo renders
            gl.DrawElements(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                0 as *const std::os::raw::c_void,
            ); // For ibo renders
        }

        window.poll_events();
        window.swap_buffers();
    }
}
