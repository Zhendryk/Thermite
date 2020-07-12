// Create our module using the line from the original gl-rs crate
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Export all the types in bindings
pub use bindings::*;

use std::ops::Deref;
use std::rc::Rc;

// Wrap the original Gl struct with a new, shared pointer (Rc == reference counted)
#[derive(Clone)] // Automatically implement the Clone trait on Gl, which for a Rc, just increments the reference counter
pub struct Gl {
    inner: Rc<bindings::Gl>,
}

// We need to wrap the load_with function (basically the constructor for this specific obj) so that it is instantiated as a Rc pointer
impl Gl {
    pub fn load_with<F>(load_fn: F) -> Gl
    where
        F: FnMut(&'static str) -> *const types::GLvoid,
    {
        Gl {
            inner: Rc::new(bindings::Gl::load_with(load_fn)),
        }
    }
}

// In order to not have to use gl.inner all the time, this is basically an implicit cast
impl Deref for Gl {
    type Target = bindings::Gl;
    fn deref(&self) -> &bindings::Gl {
        &self.inner
    }
}

// Avoid name collision with original gl-rs crate definition
pub use bindings::Gl as InnerGl;
