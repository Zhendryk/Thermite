use gl::{
    self,
    types::{GLsizei, GLuint},
};

/// A Vertex Array Object, an OpenGL construct which stores all of the state needed to supply vertex data.
pub struct VertexArray {
    gl: gl::Gl, // This is a reference counted pointer (C++ std::shared_pointer equivalent)
    id: GLuint,
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
        }
    }

    /// Returns the OpenGL GLuint id of this `VertexArray`
    pub fn id(&self) -> GLuint {
        self.id
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
}

impl Drop for VertexArray {
    // Need to delete the vertex array from OpenGL upon deallocation
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1 as GLsizei, &self.id);
        }
    }
}
