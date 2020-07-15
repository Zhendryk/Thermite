extern crate image;
use gl::{
    self,
    types::{GLint, GLsizei},
};
use image::GenericImageView;
use std::os::raw::c_void;

pub struct Texture {
    id: u32,
    gl: gl::Gl,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Texture {
    pub fn new(gl: &gl::Gl, path: &std::path::PathBuf) -> Self {
        // TODO: Match and handle error here, or do unwrap_or, etc.
        let img = image::open(path).unwrap();
        let mut id = 0;
        unsafe {
            gl.GenTextures(1, &mut id);
        }
        let (width, height) = img.dimensions();
        Texture {
            id: id,
            gl: gl.clone(),
            width: width,
            height: height,
            data: img.to_bytes(), // TODO: This should only be a borrow until we generate the texture, and then we can free this data
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn unbind(&self) {}

    pub fn generate(&self) {
        unsafe {
            self.gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as GLint,
                self.width as GLsizei,
                self.height as GLsizei,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                self.data.as_ptr() as *const c_void,
            );
            self.gl.GenerateMipmap(gl::TEXTURE_2D);
        }
    }
}
