use gl::{
    self,
    types::{GLenum, GLfloat, GLint, GLsizei, GLuint},
};
extern crate image;
use image::{GenericImageView, ImageError};
use std::os::raw::c_void;

/// Allows for setting OpenGL texture parameter values, wraps `glTexParameter<type>`
pub trait TextureParameterType {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl);
}

impl TextureParameterType for u32 {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe {
            gl.TexParameteri(texture_type, param_name, *self as GLint);
        }
    }
}

impl TextureParameterType for i32 {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe {
            gl.TexParameteri(texture_type, param_name, *self as GLint);
        }
    }
}

impl TextureParameterType for f32 {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe {
            gl.TexParameterf(texture_type, param_name, *self as GLfloat);
        }
    }
}

impl TextureParameterType for [i32] {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe {
            gl.TexParameteriv(texture_type, param_name, &self[0]);
        }
    }
}

impl TextureParameterType for [f32] {
    fn set_texture_parameter(&self, texture_type: GLenum, param_name: GLenum, gl: &gl::Gl) {
        unsafe {
            gl.TexParameterfv(texture_type, param_name, &self[0]);
        }
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
    data: Vec<u8>,
    gl: gl::Gl,
}

impl Texture {
    /// Create a new `Texture`
    ///
    /// ### Parameters
    ///
    /// - `path`: The path to the asset to create a `Texture` out of
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
        path: &std::path::PathBuf,
        target: gl::types::GLenum,
        internal_format: gl::types::GLenum,
        format: gl::types::GLenum,
        gl: &gl::Gl,
    ) -> Result<Texture, ImageError> {
        // TODO: Load texture from `Resource`
        let img = image::open(path)?;
        let mut id = 0;
        unsafe {
            gl.GenTextures(1, &mut id);
        }
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
            data: img.to_bytes(), // TODO: This should only be a borrow until we generate the texture, and then we can free this data
            gl: gl.clone(),
        })
    }

    /// Bind this texture to its target
    pub fn bind(&self) {
        unsafe {
            self.gl.BindTexture(self.target, self.id);
        }
    }

    /// Set a texture parameter on this `Texture` of the given `param_name` and `param_value`
    pub fn set_texture_parameter<T: TextureParameterType>(
        &self,
        param_name: gl::types::GLenum,
        param_value: T,
    ) {
        param_value.set_texture_parameter(self.target, param_name, &self.gl);
    }

    /// Generate the OpenGL texture for this `Texture`
    pub fn generate(&self) {
        self.generate_texture_with_optional_mipmap(false);
    }

    /// Generate the OpenGL texture for this `Texture`, along with its associated mipmap
    pub fn generate_with_mipmap(&self) {
        self.generate_texture_with_optional_mipmap(true);
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
                    self.data.as_ptr() as *const c_void,
                );
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
                    self.data.as_ptr() as *const c_void,
                );
            },
            _ => {
                println!("Unsupported texture type!");
            }
        }
        if gen_mipmap {
            unsafe {
                self.gl.GenerateMipmap(self.target);
            }
        }
    }
}
