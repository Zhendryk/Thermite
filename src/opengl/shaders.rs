use crate::resources;
use gl::{self, types::*};
use std::{
    self,
    ffi::{CStr, CString},
};

// Shader types
const SHADER_EXT: [(&str, GLenum); 2] =
    [(".vert", gl::VERTEX_SHADER), (".frag", gl::FRAGMENT_SHADER)];

/// Extension to primitive types which support OpenGL shader uniform variables
pub trait ShaderUniformType {
    fn set_uniform(&self, program_id: &gl::types::GLuint, name: &str, gl: &gl::Gl);
}

impl ShaderUniformType for bool {
    fn set_uniform(&self, program_id: &gl::types::GLuint, name: &str, gl: &gl::Gl) {
        unsafe {
            gl.Uniform1i(
                gl.GetUniformLocation(*program_id, name.as_ptr() as *const GLchar),
                *self as GLint,
            );
        }
    }
}

impl ShaderUniformType for u32 {
    fn set_uniform(&self, program_id: &gl::types::GLuint, name: &str, gl: &gl::Gl) {
        unsafe {
            gl.Uniform1i(
                gl.GetUniformLocation(*program_id, name.as_ptr() as *const GLchar),
                *self as GLint,
            );
        }
    }
}

impl ShaderUniformType for i32 {
    fn set_uniform(&self, program_id: &gl::types::GLuint, name: &str, gl: &gl::Gl) {
        unsafe {
            gl.Uniform1i(
                gl.GetUniformLocation(*program_id, name.as_ptr() as *const GLchar),
                *self as GLint,
            );
        }
    }
}

impl ShaderUniformType for f32 {
    fn set_uniform(&self, program_id: &gl::types::GLuint, name: &str, gl: &gl::Gl) {
        unsafe {
            gl.Uniform1f(
                gl.GetUniformLocation(*program_id, name.as_ptr() as *const GLchar),
                *self as GLfloat,
            );
        }
    }
}

// Errors relating to `Shader`s and `ShaderProgram`s
#[derive(Debug)]
pub enum ShaderError {
    ResourceLoadError {
        name: String,
        inner: resources::ResourceError,
    },
    CannotDetermineShaderTypeForResource {
        name: String,
    },
    CompileError {
        name: String,
        message: String,
    },
    LinkError {
        name: String,
        message: String,
    },
}

/// A `Shader` to use in an OpenGL `ShaderProgram`
pub struct Shader {
    gl: gl::Gl, // This is a reference counted pointer (C++ std::shared_pointer equivalent)
    id: GLuint,
}

impl Shader {
    /// Creates a new `Shader` using the given `filename` inside the given `Resource`, if it exists
    /// ### Parameters
    ///
    /// - `res`: A `Resource` pointing to the directory where the `Shader` file is stored on disk
    /// - `filename`: The file name of this shader
    /// - `gl`: Reference counted pointer to the current OpenGL context
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: A `Shader` to use within a `ShaderProgram`
    /// - `Err`: A `ShaderError` with a name (`filename`) and a message of what went wrong during `Shader` creation
    pub fn new(
        res: &resources::Resource,
        filename: &str,
        gl: &gl::Gl,
    ) -> Result<Shader, ShaderError> {
        // Get the type of this shader by comparing it to our map of `Shader` types
        let shader_type = SHADER_EXT
            .iter()
            .find(|&&(file_extension, _)| filename.ends_with(file_extension))
            .map(|&(_, kind)| kind)
            .ok_or_else(|| ShaderError::CannotDetermineShaderTypeForResource {
                name: filename.into(),
            })?;
        // Load the data from the file containing the `Shader` source code into memory
        let shader_source = res
            .load(filename)
            .map_err(|e| ShaderError::ResourceLoadError {
                name: filename.into(),
                inner: e,
            })?;
        // Finally, create our shader using the source code data and type
        Shader::from_source(&shader_source, shader_type, gl).map_err(|message| {
            ShaderError::CompileError {
                name: filename.into(),
                message,
            }
        })
    }

    /// Returns an immutable reference to the `GLuint` id of this `Shader`
    pub fn id(&self) -> &GLuint {
        &self.id
    }

    /// Create a `Shader` of the given `kind` using the given `source`
    ///
    /// ### Parameters
    ///
    /// - `source`: The source code of the shader
    /// - `kind`: The kind of `Shader` to create, e.g. Vertex, Fragment, etc.
    /// - `gl`: Reference counted pointer to the current OpenGL context
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: A `Shader` to use within a `ShaderProgram`
    /// - `Err`: A `String` message with the OpenGL `Shader` info log
    fn from_source(source: &CStr, kind: GLenum, gl: &gl::Gl) -> Result<Shader, String> {
        // Create the shader id
        let id = unsafe { gl.CreateShader(kind) };
        // Hook up a pointer to the shader's source code and compile it
        unsafe {
            gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl.CompileShader(id);
        }
        // Check the success (or lack thereof) of the compilation
        let mut success = gl::FALSE as GLint;
        unsafe {
            gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }
        if success != gl::TRUE as GLint {
            let mut len: GLint = 0;
            unsafe {
                gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let buffer: CString = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl.GetShaderInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_ptr() as *mut GLchar,
                );
            }
            return Err(buffer.to_string_lossy().into_owned());
        }
        Ok(Shader {
            gl: gl.clone(),
            id: id,
        })
    }
}

impl Drop for Shader {
    // Need to delete the shader from OpenGL on deallocation
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

/// An OpenGL `ShaderProgram` to use for rendering
pub struct ShaderProgram {
    gl: gl::Gl, // This is a reference counted pointer (C++ std::shared_pointer equivalent)
    id: GLuint,
}

impl ShaderProgram {
    /// Creates a new OpenGL `ShaderProgram` using all of the shaders in the given `Resource` which share the `program_name`
    /// ### Parameters
    ///
    /// - `res`: A `Resource` pointing to the directory where the `Shader`s for this `ShaderProgram` are stored on disk
    /// - `program_name`: The name of this program, used to identify all of the `Shader`s used within it
    /// - `gl`: Reference counted pointer to the current OpenGL context
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: An OpenGL `ShaderProgram` to use for rendering
    /// - `Err`: A `ShaderError` with a name (`program_name`) and a message of what went wrong during `ShaderProgram` creation
    pub fn new(
        res: &resources::Resource,
        program_name: &str,
        gl: &gl::Gl,
    ) -> Result<ShaderProgram, ShaderError> {
        // When creating a shader program this way, it is assumed all shaders used in the program have the following naming scheme: program_name.ext
        let shader_filenames = SHADER_EXT
            .iter()
            .map(|(file_extension, _)| format!("{}{}", program_name, file_extension))
            .collect::<Vec<String>>();
        // Load every type of shader for this shader program
        let shaders = shader_filenames
            .iter()
            .map(|filename| Shader::new(res, filename, gl))
            .collect::<Result<Vec<Shader>, ShaderError>>()?;
        // Finally, create the program using these shaders
        ShaderProgram::from_shaders(&shaders[..], gl).map_err(|message| ShaderError::LinkError {
            name: program_name.into(),
            message,
        })
    }

    /// Returns an immutable reference to the `GLuint` id of this `ShaderProgram`
    pub fn id(&self) -> &GLuint {
        &self.id
    }

    /// Installs this `ShaderProgram` as part of the current OpenGL rendering state
    pub fn use_program(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }

    /// Set the value of a uniform variable in the current shader program stack, if it exists
    pub fn set_uniform<T: ShaderUniformType>(&self, name: &str, value: T) {
        value.set_uniform(&self.id, name, &self.gl);
    }

    /// Create a shader program with the given list of shaders
    /// ### Parameters
    ///
    /// - `using_shaders`: A list of the `Shader` structs to link together into a `ShaderProgram`
    /// - `gl`: Reference counted pointer to the current OpenGL context
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: An OpenGL `ShaderProgram` to use for rendering
    /// - `Err`: A `String` message with the OpenGL `ShaderProgram` info log
    fn from_shaders(using_shaders: &[Shader], gl: &gl::Gl) -> Result<ShaderProgram, String> {
        // Create our program id
        let id = unsafe { gl.CreateProgram() };
        // Attach all of our shaders to the program
        for shader in using_shaders {
            unsafe {
                gl.AttachShader(id, *shader.id());
            }
        }
        // Link the program
        unsafe {
            gl.LinkProgram(id);
        }
        // Check the success (or lack thereof) of the linkage
        let mut success = gl::FALSE as GLint;
        unsafe {
            gl.GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }
        if success != gl::TRUE as GLint {
            let mut len: GLint = 0;
            unsafe {
                gl.GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error: CString = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl.GetProgramInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            }
            return Err(error.to_string_lossy().into_owned());
        }
        // Detach all of the shaders after successfully linking the program, so they can be deleted when they go out of scope
        for shader in using_shaders {
            unsafe {
                // gl.DeleteShader will not delete if it is attached to the program
                // Since we've already linked the program, we can detach them
                gl.DetachShader(id, *shader.id());
            }
        }
        Ok(ShaderProgram { gl: gl.clone(), id })
    }
}

impl Drop for ShaderProgram {
    // Need to delete the ShaderProgram from OpenGL on deallocation
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // Allocate a buffer (+ 1 for null termination character)
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // Fill it with whitespace
    buffer.extend([b' '].iter().cycle().take(len as usize));
    // Convert to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
