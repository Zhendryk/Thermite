// use gl;
// use glfw::{self, WindowHint, WindowMode};
// extern crate nalgebra_glm as glm;
// use std::path::Path;

// pub mod opengl;
// use opengl::{
//     buffer_layout::{BufferComponent, BufferComponentType, BufferLayout},
//     camera::Camera,
//     // index_buffer::IndexBuffer,
//     shaders,
//     texture::Texture,
//     vertex_array::VertexArray,
//     vertex_buffer::VertexBuffer,
//     window::GLFWWindow,
// };

// pub mod resources;
// use resources::Resource;

// const WIDTH: u32 = 1920;
// const HEIGHT: u32 = 1080;
// const ASPECT: f32 = WIDTH as f32 / HEIGHT as f32;
// const CENTER_X: f64 = WIDTH as f64 / 2.0;
// const CENTER_Y: f64 = HEIGHT as f64 / 2.0;

// fn main() {
//     // Create a GLFW window
//     let hints = vec![
//         WindowHint::ContextVersion(3, 3),
//         WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core),
//         WindowHint::OpenGlForwardCompat(true),
//     ];
//     let mut window = GLFWWindow::new(
//         WIDTH,
//         HEIGHT,
//         "Thermite Engine v0.1.0",
//         WindowMode::Windowed,
//         glfw::FAIL_ON_ERRORS,
//         Option::from(hints),
//     )
//     .expect("Failed to create GLFW window");

//     // Make our OpenGL context current and set some window callbacks
//     window.make_context_current();
//     window.set_key_polling(true);
//     window.set_framebuffer_size_polling(true);
//     window.set_scroll_polling(true);
//     window.set_cursor_pos_polling(true);
//     window.set_cursor_mode(glfw::CursorMode::Disabled);

//     // Load all of our OpenGL function pointers into the gl wrapper object
//     let gl = window.load_opengl_fn_ptrs();

//     // Configure global OpenGL state
//     unsafe {
//         gl.Enable(gl::DEPTH_TEST);
//     }

//     // Grab our shader resource and create a shader program with it
//     let shaders = Resource::new(Path::new("assets/shaders"))
//         .expect("Could not create resource from assets/shaders");
//     let shader_program = shaders::ShaderProgram::new(&shaders, "main", &gl)
//         .expect("Could not create shader program.");

//     // Create the layout of our rendering data
//     let vertices: [f32; 180] = [
//         // Face 1
//         // pos            // tex
//         -0.5, -0.5, -0.5, 0.0, 0.0, // p1
//         0.5, -0.5, -0.5, 1.0, 0.0, // p2
//         0.5, 0.5, -0.5, 1.0, 1.0, // p3
//         0.5, 0.5, -0.5, 1.0, 1.0, // p4
//         -0.5, 0.5, -0.5, 0.0, 1.0, // p5
//         -0.5, -0.5, -0.5, 0.0, 0.0, // p6
//         // Face 2
//         // pos           // tex
//         -0.5, -0.5, 0.5, 0.0, 0.0, // p1
//         0.5, -0.5, 0.5, 1.0, 0.0, // p2
//         0.5, 0.5, 0.5, 1.0, 1.0, // p3
//         0.5, 0.5, 0.5, 1.0, 1.0, // p4
//         -0.5, 0.5, 0.5, 0.0, 1.0, // p5
//         -0.5, -0.5, 0.5, 0.0, 0.0, // p6
//         // Face 3
//         // pos          // tex
//         -0.5, 0.5, 0.5, 1.0, 0.0, // p1
//         -0.5, 0.5, -0.5, 1.0, 1.0, // p2
//         -0.5, -0.5, -0.5, 0.0, 1.0, // p3
//         -0.5, -0.5, -0.5, 0.0, 1.0, // p4
//         -0.5, -0.5, 0.5, 0.0, 0.0, // p5
//         -0.5, 0.5, 0.5, 1.0, 0.0, // p6
//         // Face 4
//         // pos         // tex
//         0.5, 0.5, 0.5, 1.0, 0.0, // p1
//         0.5, 0.5, -0.5, 1.0, 1.0, // p2
//         0.5, -0.5, -0.5, 0.0, 1.0, // p3
//         0.5, -0.5, -0.5, 0.0, 1.0, // p4
//         0.5, -0.5, 0.5, 0.0, 0.0, // p5
//         0.5, 0.5, 0.5, 1.0, 0.0, // p6
//         // Face 5
//         // pos            // tex
//         -0.5, -0.5, -0.5, 0.0, 1.0, // p1
//         0.5, -0.5, -0.5, 1.0, 1.0, // p2
//         0.5, -0.5, 0.5, 1.0, 0.0, // p3
//         0.5, -0.5, 0.5, 1.0, 0.0, // p4
//         -0.5, -0.5, 0.5, 0.0, 0.0, // p5
//         -0.5, -0.5, -0.5, 0.0, 1.0, // p6
//         // Face 6
//         // pos           // tex
//         -0.5, 0.5, -0.5, 0.0, 1.0, // p1
//         0.5, 0.5, -0.5, 1.0, 1.0, // p2
//         0.5, 0.5, 0.5, 1.0, 0.0, // p3
//         0.5, 0.5, 0.5, 1.0, 0.0, // p4
//         -0.5, 0.5, 0.5, 0.0, 0.0, // p5
//         -0.5, 0.5, -0.5, 0.0, 1.0, // p6
//     ];

//     // Create 10 cubes
//     let cube_positions: [glm::Vec3; 10] = [
//         glm::vec3(0.0, 0.0, -2.0),
//         glm::vec3(2.0, 5.0, -15.0),
//         glm::vec3(-1.5, -2.2, -2.5),
//         glm::vec3(-3.8, -2.0, -12.3),
//         glm::vec3(2.4, -0.4, -3.5),
//         glm::vec3(-1.7, 3.0, -7.5),
//         glm::vec3(1.3, -2.0, -2.5),
//         glm::vec3(1.5, 2.0, -2.5),
//         glm::vec3(1.5, 0.2, -1.5),
//         glm::vec3(-1.3, 1.0, -1.5),
//     ];

//     // let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
//     let positions = BufferComponent::new(
//         String::from("positions"),
//         BufferComponentType::Float3,
//         false,
//     );
//     let texture_coords = BufferComponent::new(
//         String::from("tex-coords"),
//         BufferComponentType::Float2,
//         false,
//     );
//     let layout = BufferLayout::new(&mut [positions, texture_coords]);
//     // Create our vao, vbo and ibo
//     let mut vao = VertexArray::new(&gl);
//     let vbo = VertexBuffer::new(&gl, layout);
//     vbo.bind();
//     vbo.buffer_data(&vertices, gl::STATIC_DRAW);
//     vao.add_vertex_buffer(vbo);
//     // let ibo = IndexBuffer::new(&gl);
//     // ibo.bind();
//     // ibo.buffer_data(&indices, gl::STATIC_DRAW);
//     // vao.set_index_buffer(ibo);
//     vao.unbind();

//     // Create our textures and bind them to the shader program
//     let textures = Resource::new(Path::new("assets/textures"))
//         .expect("Could not create resource from assets/textures");
//     let wall_texture = Texture::new("wall.jpg", &textures, gl::TEXTURE_2D, gl::RGB, gl::RGB, &gl)
//         .expect("Could not load wall texture");
//     wall_texture.bind();
//     wall_texture.set_texture_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
//     wall_texture.set_texture_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
//     wall_texture.set_texture_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR);
//     wall_texture.set_texture_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
//     wall_texture.generate_with_mipmap();
//     let mut face_texture = Texture::new(
//         "awesomeface.png",
//         &textures,
//         gl::TEXTURE_2D,
//         gl::RGBA,
//         gl::RGBA,
//         &gl,
//     )
//     .expect("Could not load face texture");
//     face_texture.bind();
//     face_texture.set_texture_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
//     face_texture.set_texture_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
//     face_texture.set_texture_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR);
//     face_texture.set_texture_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
//     face_texture.flip_vertically();
//     face_texture.generate_with_mipmap();
//     shader_program.use_program();
//     shader_program.set_uniform("texture1", 0);
//     shader_program.set_uniform("texture2", 1);

//     // Uncomment this to draw wireframe polygons
//     // unsafe {
//     //     gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
//     // }

//     let mut camera = Camera::new();

//     let mut delta_time = 0.0;
//     let mut last_frame = 0.0;
//     let mut last_x: f64 = CENTER_X;
//     let mut last_y: f64 = CENTER_Y;
//     let mut first_mouse: bool = true;

//     while !window.should_close() {
//         let current_frame = window.get_time();
//         delta_time = current_frame - last_frame;
//         last_frame = current_frame;
//         window.process_events(
//             &gl,
//             &(delta_time as f32),
//             &mut last_x,
//             &mut last_y,
//             &mut first_mouse,
//             &mut camera,
//         );

//         unsafe {
//             // Clear
//             gl.ClearColor(0.2, 0.3, 0.3, 1.0);
//             gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

//             // Bind
//             gl.ActiveTexture(gl::TEXTURE0);
//             wall_texture.bind();
//             gl.ActiveTexture(gl::TEXTURE1);
//             face_texture.bind();

//             // shader program work
//             shader_program.use_program();
//             let view = camera.get_view_matrix();
//             let projection =
//                 glm::perspective(ASPECT, radians(*camera.fov() as f64) as f32, 0.1, 100.0);
//             shader_program.set_uniform("view", glm::value_ptr(&view));
//             shader_program.set_uniform("projection", glm::value_ptr(&projection));

//             // bind the data
//             vao.bind();
//             for i in 0..10 {
//                 let mut model = glm::Mat4::identity();
//                 model = glm::translate(&model, &cube_positions[i]);
//                 let angle: f64 = 20.0 * i as f64;
//                 let norm = angle / 20.0;
//                 model = glm::rotate(
//                     &model,
//                     (window.get_time() * radians(angle)) as f32,
//                     &glm::vec3(norm as f32, 1.0, 0.0),
//                 );
//                 shader_program.set_uniform("model", glm::value_ptr(&model));

//                 // Draw
//                 // For non-ibo renders
//                 gl.DrawArrays(gl::TRIANGLES, 0, 36);

//                 // For ibo renders
//                 // gl.DrawElements(
//                 //     gl::TRIANGLES,
//                 //     6,
//                 //     gl::UNSIGNED_INT,
//                 //     0 as *const std::os::raw::c_void,
//                 // );
//             }
//         }

//         window.poll_events();
//         window.swap_buffers();
//     }
// }

// fn radians(deg: f64) -> f64 {
//     deg * (std::f64::consts::PI / 180.0)
// }
