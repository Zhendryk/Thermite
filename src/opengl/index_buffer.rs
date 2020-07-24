use gl::{
    self,
    types::{GLenum, GLsizei, GLsizeiptr, GLuint},
};
use std::os::raw::c_void;

/// An OpenGL Index (Element) Buffer to allow for indexed drawing on render calls which have overlapping vertex data
pub struct IndexBuffer {
    gl: gl::Gl, // This is a reference counted pointer (C++ std::shared_pointer equivalent)
    id: GLuint,
}

impl IndexBuffer {
    /// Creates a new Index Buffer Object, and OpenGL construct which allows for indexed drawing on render calls which have overlapping vertex data
    ///
    /// ### Parameters
    ///
    /// - `gl`: Reference counted pointer to the current OpenGL context
    ///
    /// ### Returns
    ///
    /// A newly initialized `IndexBuffer` (unbound)
    pub fn new(gl: &gl::Gl) -> IndexBuffer {
        let mut id: GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut id);
        }
        IndexBuffer {
            gl: gl.clone(),
            id: id,
        }
    }

    /// Returns a reference to the OpenGL GLuint id of this `IndexBuffer`
    pub fn id(&self) -> &GLuint {
        &self.id
    }

    /// Bind this `IndexBuffer` object to the current OpenGL context
    pub fn bind(&self) {
        // TODO: Look into this vs ARRAY_BUFFER, as the former requires a VAO
        unsafe { self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id) }
    }

    /// Unbind this `IndexBuffer` object from the current OpenGL context
    pub fn unbind(&self) {
        unsafe { self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0) }
    }

    /// Create and initialize this `IndexBuffer`'s data store with the given data (`indices`) in the given `mode`
    ///
    /// ### Parameters
    ///
    /// - `indices`: The vertex index data to buffer into this `IndexBuffer`
    /// - `mode`: The OpenGL drawing mode to use for this `IndexBuffer`, e.g. `GL_STATIC_DRAW`, `GL_DYNAMIC_DRAW`, etc.
    pub fn buffer_data(&self, indices: &[u32], mode: GLenum) {
        unsafe {
            self.gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<GLuint>()) as GLsizeiptr,
                &indices[0] as *const u32 as *const c_void,
                mode,
            )
        }
    }
}

impl Drop for IndexBuffer {
    // Need to delete the buffer from OpenGL upon deallocation
    fn drop(&mut self) {
        unsafe { self.gl.DeleteBuffers(1 as GLsizei, &self.id) }
    }
}
