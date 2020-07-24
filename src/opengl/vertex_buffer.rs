use crate::opengl::buffer_layout::BufferLayout;
use gl::{
    self,
    types::{GLenum, GLfloat, GLsizei, GLsizeiptr, GLuint},
};
use std::os::raw::c_void;

/// A `VertexBuffer` to store raw vertex information for graphical rendering
pub struct VertexBuffer {
    gl: gl::Gl, // This is a reference counted pointer (C++ std::shared_pointer equivalent)
    id: GLuint,
    layout: BufferLayout,
}

impl VertexBuffer {
    /// Create a new Vertex Buffer Object (vbo)
    ///
    /// ### Parameters
    ///
    /// - `gl`: Reference counted pointer to the current OpenGL context
    /// - `layout`: The `BufferLayout` detailing the layout of the data (stride, offsets, etc.)
    ///
    /// ### Returns
    ///
    /// A newly initialized `VertexBuffer` (unbound)
    pub fn new(gl: &gl::Gl, layout: BufferLayout) -> VertexBuffer {
        let mut id: GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut id);
        }
        VertexBuffer {
            gl: gl.clone(),
            id: id,
            layout: layout,
        }
    }

    /// Returns a reference to the OpenGL GLuint id of this `VertexBuffer`
    pub fn id(&self) -> &GLuint {
        &self.id
    }

    /// Returns a reference to the `BufferLayout` of the data within this `VertexBuffer`
    pub fn layout(&self) -> &BufferLayout {
        &self.layout
    }

    /// Bind this `VertexBuffer` to the OpenGL `GL_ARRAY_BUFFER` target
    pub fn bind(&self) {
        unsafe { self.gl.BindBuffer(gl::ARRAY_BUFFER, self.id) }
    }

    /// Unbind this `VertexBuffer` from the OpenGL `GL_ARRAY_BUFFER` target
    pub fn unbind(&self) {
        unsafe { self.gl.BindBuffer(gl::ARRAY_BUFFER, 0) }
    }

    /// Create and initialize this `VertexBuffer`'s data store with the given data (`vertices`) in the given `mode`
    ///
    /// ### Parameters
    ///
    /// - `vertices`: The vertex data to buffer into this `VertexBuffer`
    /// - `mode`: The OpenGL drawing mode to use for this `VertexBuffer`, e.g. `GL_STATIC_DRAW`, `GL_DYNAMIC_DRAW`, etc.
    pub fn buffer_data(&self, vertices: &[f32], mode: GLenum) {
        unsafe {
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                &vertices[0] as *const f32 as *const c_void,
                mode,
            )
        }
    }
}

impl Drop for VertexBuffer {
    // Need to delete the buffer from OpenGL upon deallocation
    fn drop(&mut self) {
        unsafe { self.gl.DeleteBuffers(1 as GLsizei, &self.id) }
    }
}
