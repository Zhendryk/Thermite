use gfx_hal::{self, device::Device, pso::ShaderStageFlags, Backend};
use thermite_core::resources;

#[derive(Debug, Copy, Clone)]
pub enum ShaderType {
    Spirv,
    Glsl,
    Hlsl,
    Metal,
}

// Shader types
const SHADER_EXT: [(&str, ShaderType); 4] = [
    (".spv", ShaderType::Spirv),
    (".glsl", ShaderType::Glsl),
    (".metal", ShaderType::Metal),
    (".hlsl", ShaderType::Hlsl),
];

#[derive(Debug)]
pub enum ShaderError {
    ResourceLoadError {
        name: String,
        inner: resources::ResourceError,
    },
    CannotDetermineShaderTypeForResource {
        name: String,
    },
    UnsupportedShaderType {
        name: String,
        unsupported_type: String,
    },
    CompileError {
        name: String,
        message: String,
    },
    LinkError {
        name: String,
        message: String,
    },
    SpirvReadError {
        name: String,
        inner: std::io::Error,
    },
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderError::ResourceLoadError { name, inner } => {
                write!(f, "{:?} ({}): {:?}", self, name, inner)
            }
            ShaderError::CannotDetermineShaderTypeForResource { name } => {
                write!(f, "{:?} ({})", self, name)
            }
            ShaderError::UnsupportedShaderType {
                name,
                unsupported_type,
            } => write!(f, "{:?} ({}): {}", self, name, unsupported_type),
            ShaderError::CompileError { name, message } => {
                write!(f, "{:?} ({}): {}", self, name, message)
            }
            ShaderError::LinkError { name, message } => {
                write!(f, "{:?} ({}): {}", self, name, message)
            }
            ShaderError::SpirvReadError { name, inner } => {
                write!(f, "{:?} ({}): {:?}", self, name, inner)
            }
        }
    }
}

// TODO: Docstrings, comments, maybe creating shader modules from this module?
pub struct Shader {
    kind: ShaderType,
    filename: String,
    stage: ShaderStageFlags,
    entry: String,
    spirv: Vec<u32>,
}

impl Shader {
    pub fn new(
        res: &resources::Resource,
        filename: &str,
        stage: ShaderStageFlags,
        entry: &str,
    ) -> Result<Shader, ShaderError> {
        let shader_type = SHADER_EXT
            .iter()
            .find(|&&(ext, _)| filename.ends_with(ext))
            .map(|&(_, kind)| kind)
            .ok_or_else(|| ShaderError::CannotDetermineShaderTypeForResource {
                name: filename.into(),
            })?;
        let bytecode =
            res.load_to_bytes(filename, false)
                .map_err(|e| ShaderError::ResourceLoadError {
                    name: filename.into(),
                    inner: e,
                })?;
        let spirv = gfx_hal::pso::read_spirv(std::io::Cursor::new(&bytecode)).map_err(|e| {
            ShaderError::SpirvReadError {
                name: filename.into(),
                inner: e,
            }
        })?;
        Ok(Shader {
            kind: shader_type,
            filename: filename.to_owned(),
            stage: stage,
            entry: entry.to_owned(),
            spirv: spirv,
        })
    }

    pub unsafe fn module<B>(
        &self,
        logical_device: &B::Device,
    ) -> Result<B::ShaderModule, ShaderError>
    where
        B: Backend,
    {
        logical_device
            .create_shader_module(&self.spirv)
            .map_err(|e| ShaderError::CompileError {
                name: self.filename.clone(),
                message: format!("{:?}", e),
            })
    }
}
