use crate::opengl::{buffer_layout, index_buffer, vertex_buffer};
use gl::{
    self,
    types::{GLboolean, GLint, GLsizei, GLuint},
};
use std::os::raw::c_void;

/// A Vertex Array Object, an OpenGL construct which stores all of the state needed to supply vertex data.
pub struct VertexArray {
    gl: gl::Gl, // This is a reference counted pointer (C++ std::shared_pointer equivalent)
    id: GLuint,
    vertex_buffers: Vec<vertex_buffer::VertexBuffer>,
    vb_index: u32,
    index_buffer: Option<index_buffer::IndexBuffer>,
}

impl VertexArray {
    /// Creates a new Vertex Array Object, an OpenGL construct which stores all of the state needed to supply vertex data.
    ///
    /// # Parameters
    ///
    /// - `gl`: Reference counted pointer to the current OpenGL context
    ///
    /// # Returns
    ///
    /// A newly initialized `VertexArray` (unbound)
    pub fn new(gl: &gl::Gl) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut id);
        }
        VertexArray {
            gl: gl.clone(),
            id: id,
            vertex_buffers: Vec::new(),
            vb_index: 0,
            index_buffer: Option::default(),
        }
    }

    /// Returns the OpenGL GLuint id of this `VertexArray`
    pub fn id(&self) -> &GLuint {
        &self.id
    }

    /// Bind this `VertexArray` object to the current OpenGL context
    pub fn bind(&self) {
        unsafe {
            self.gl.BindVertexArray(self.id);
        }
    }

    /// Unbind this `VertexArray` object from the current OpenGL context
    pub fn unbind(&self) {
        unsafe {
            self.gl.BindVertexArray(0);
        }
    }

    pub fn add_vertex_buffer(&mut self, vbo: vertex_buffer::VertexBuffer) {
        self.bind();
        vbo.bind();
        let layout = vbo.layout();
        for component in layout.components() {
            match component.kind() {
                buffer_layout::BufferComponentType::Float3 => {
                    unsafe {
                        self.gl.EnableVertexAttribArray(self.vb_index);
                        self.gl.VertexAttribPointer(
                            self.vb_index as GLuint,
                            *component.count() as GLint,
                            gl::FLOAT,
                            *component.normalized() as GLboolean,
                            *layout.stride() as GLsizei,
                            *component.offset() as *const c_void,
                        );
                    }
                    self.vb_index += 1;
                }
                _ => println!("Unsupported BufferComponentType!"),
            }
        }
        self.vertex_buffers.push(vbo);
    }

    pub fn set_index_buffer(&mut self, ibo: index_buffer::IndexBuffer) {
        self.bind();
        ibo.bind();
        self.index_buffer = Option::from(ibo);
    }
}

impl Drop for VertexArray {
    // Need to delete the vertex array from OpenGL upon deallocation
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1 as GLsizei, &self.id);
        }
    }
}
