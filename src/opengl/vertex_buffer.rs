use gl::{
    self,
    types::{GLfloat, GLsizei, GLsizeiptr, GLuint},
};
use std::os::raw::c_void;

#[derive(Default)]
pub struct VertexBuffer {
    // gl: gl::Gl, // This is a reference counted pointer
    id: u32,
}

impl VertexBuffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn bind(&mut self, gl: &gl::Gl) {
        unsafe {
            gl.BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn unbind(&self, gl: &gl::Gl) {
        unsafe {
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn buffer_data(&self, gl: &gl::Gl, vertices: &[f32], mode: gl::types::GLenum) {
        unsafe {
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                &vertices[0] as *const f32 as *const c_void,
                mode,
            );
        }
    }
}

// impl Default for VertexBuffer {
//     fn default(&self) -> Self {
//         VertexBuffer {
//             gl: gl::Gl {},
//             id: 0,
//         }
//     }
// }

// impl Drop for VertexBuffer {
//     fn drop(&mut self) {
//         unsafe {
//             self.gl
//                 .DeleteBuffers(1 as GLsizei, &self.renderer_id as *const GLuint);
//         }
//     }
// }
