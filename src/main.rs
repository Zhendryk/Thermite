use gl;
use glfw::{self, WindowHint, WindowMode};
use std::path::Path;

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
    // Create a GLFW window
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

    // Make our OpenGL context current and set some window callbacks
    window.make_context_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // Load all of our OpenGL function pointers into the gl wrapper object
    let gl = window.load_opengl_fn_ptrs();

    // Grab our shader resource and create a shader program with it
    let shaders = Resource::new(Path::new("assets/shaders"))
        .expect("Could not create resource from assets/shaders");
    let shader_program = shaders::ShaderProgram::new(&shaders, "uniformtextured", &gl)
        .expect("Could not create shader program.");

    // Create the layout of our rendering data
    let vertices: [f32; 32] = [
        // positions    // colors     // texture coords
        0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
        0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
        -0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // top left
    ];
    let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
    let positions = BufferComponent::new(
        String::from("positions"),
        BufferComponentType::Float3,
        false,
    );
    let colors = BufferComponent::new(String::from("colors"), BufferComponentType::Float3, false);
    let texture_coords = BufferComponent::new(
        String::from("tex-coords"),
        BufferComponentType::Float2,
        false,
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

    // Create our textures and bind them to the shader program
    let textures = Resource::new(Path::new("assets/textures"))
        .expect("Could not create resource from assets/textures");
    let wall_texture = Texture::new("wall.jpg", &textures, gl::TEXTURE_2D, gl::RGB, gl::RGB, &gl)
        .expect("Could not load wall texture");
    wall_texture.bind();
    wall_texture.set_texture_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    wall_texture.set_texture_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    wall_texture.set_texture_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR);
    wall_texture.set_texture_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    wall_texture.generate_with_mipmap();
    let mut face_texture = Texture::new(
        "awesomeface.png",
        &textures,
        gl::TEXTURE_2D,
        gl::RGBA,
        gl::RGBA,
        &gl,
    )
    .expect("Could not load face texture");
    face_texture.bind();
    face_texture.set_texture_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    face_texture.set_texture_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    face_texture.set_texture_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR);
    face_texture.set_texture_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    face_texture.flip_vertically();
    face_texture.generate_with_mipmap();
    shader_program.use_program();
    shader_program.set_uniform("texture1", 0);
    shader_program.set_uniform("texture2", 1);

    // Uncomment this to draw wireframe polygons
    // unsafe {
    //     gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    // }

    while !window.should_close() {
        window.process_events(&gl);

        unsafe {
            // Clear
            gl.ClearColor(0.2, 0.3, 0.3, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);

            // Bind
            gl.ActiveTexture(gl::TEXTURE0);
            wall_texture.bind();
            gl.ActiveTexture(gl::TEXTURE1);
            face_texture.bind();
            shader_program.use_program();
            vao.bind();

            // Draw
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
