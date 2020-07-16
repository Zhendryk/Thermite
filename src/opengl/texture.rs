use gl::{
    self,
    types::{GLenum, GLfloat, GLint, GLsizei, GLuint},
};
extern crate image;
use crate::resources;
use image::{DynamicImage, GenericImageView, ImageError};
use std::os::raw::c_void;

/// Allows for setting OpenGL texture parameter values, wraps `glTexParameter<type>`
pub trait TextureParameterType {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl);
}

impl TextureParameterType for u32 {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe { gl.TexParameteri(texture_type, param_name, *self as GLint) }
    }
}

impl TextureParameterType for i32 {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe { gl.TexParameteri(texture_type, param_name, *self as GLint) }
    }
}

impl TextureParameterType for f32 {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe { gl.TexParameterf(texture_type, param_name, *self as GLfloat) }
    }
}

impl TextureParameterType for [i32] {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe { gl.TexParameteriv(texture_type, param_name, &self[0]) }
    }
}

impl TextureParameterType for [f32] {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe { gl.TexParameterfv(texture_type, param_name, &self[0]) }
    }
}

/// An image `Texture` to be used for graphical rendering in OpenGL
pub struct Texture {
    id: GLuint,
    target: GLenum,
    level: GLint,
    internal_format: GLenum,
    format: GLenum,
    width: u32,
    height: u32,
    depth: Option<u32>,
    img: DynamicImage,
    gl: gl::Gl,
}

impl Texture {
    /// Create a new `Texture`
    ///
    /// ### Parameters
    ///
    /// - `filename`: The name of the file to use for this texture, in the format "name.extension"
    /// - `res`: The `Resource` containing the image file to use for this `Texture`
    /// - `target`: The type of texture to create (2D, 3D, etc.)
    /// - `internal_format`: Specifies the number of color components in the texture, as a GLenum
    /// - `format`: Specifies the format of the pixel data, as a GLenum
    /// - `gl`: Reference counted pointer to the current OpenGL context
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: A newly initialized `Texture` (unbound)
    /// - `Err`: An `image::ImageError` describing what went wrong during `Texture` initialization
    pub fn new(
        filename: &str,
        res: &resources::Resource,
        target: gl::types::GLenum,
        internal_format: gl::types::GLenum,
        format: gl::types::GLenum,
        gl: &gl::Gl,
    ) -> Result<Texture, ImageError> {
        let img = image::open(res.path_for(filename))?;
        let mut id = 0;
        unsafe { gl.GenTextures(1, &mut id) }
        let (width, height) = img.dimensions();
        Ok(Texture {
            id: id,
            target: target,
            level: 0,
            internal_format: internal_format,
            format: format,
            width: width,
            height: height,
            depth: if target == gl::TEXTURE_2D {
                Option::None
            } else {
                Option::from(0)
            },
            img: img,
            gl: gl.clone(),
        })
    }

    /// Bind this texture to its target
    pub fn bind(&self) {
        unsafe { self.gl.BindTexture(self.target, self.id) }
    }

    /// Flip this texture horizontally
    pub fn flip_horizontally(&mut self) {
        self.img = self.img.fliph()
    }

    /// Flip this texture vertically
    pub fn flip_vertically(&mut self) {
        self.img = self.img.flipv()
    }

    /// Set a texture parameter on this `Texture` of the given `param_name` and `param_value`
    pub fn set_texture_parameter<T: TextureParameterType>(
        &self,
        param_name: gl::types::GLenum,
        param_value: T,
    ) {
        param_value.set_texture_parameter(self.target, param_name, &self.gl)
    }

    /// Generate the OpenGL texture for this `Texture`
    pub fn generate(&self) {
        self.generate_texture_with_optional_mipmap(false)
    }

    /// Generate the OpenGL texture for this `Texture`, along with its associated mipmap
    pub fn generate_with_mipmap(&self) {
        self.generate_texture_with_optional_mipmap(true)
    }

    /// Generate a texture of this `Texture`'s target, and optionally, the associated mipmap
    fn generate_texture_with_optional_mipmap(&self, gen_mipmap: bool) {
        match self.target {
            gl::TEXTURE_2D => unsafe {
                self.gl.TexImage2D(
                    self.target,
                    self.level,
                    self.internal_format as GLint,
                    self.width as GLsizei,
                    self.height as GLsizei,
                    0,
                    self.format,
                    gl::UNSIGNED_BYTE,
                    self.img.to_bytes().as_ptr() as *const c_void,
                )
            },
            gl::TEXTURE_3D => unsafe {
                let depth = self.depth.unwrap_or(0) as GLsizei;
                self.gl.TexImage3D(
                    self.target,
                    self.level,
                    self.internal_format as GLint,
                    self.width as GLsizei,
                    self.height as GLsizei,
                    depth,
                    0,
                    self.format,
                    gl::UNSIGNED_BYTE,
                    self.img.to_bytes().as_ptr() as *const c_void,
                )
            },
            _ => println!("Unsupported texture type!"),
        }
        if gen_mipmap {
            unsafe { self.gl.GenerateMipmap(self.target) }
        }
    }
}
