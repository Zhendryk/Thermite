use gfx_hal::{self, device::Device, pso::ShaderStageFlags, Backend};
use thermite_core::resources;

#[repr(C)] // Layout this struct in memory the same as C (and shader code) would
#[derive(Debug, Clone, Copy)]
pub struct PushConstants {
    pub transform: [[f32; 4]; 4],
}

pub fn make_transform(translate: [f32; 3], angle: f32, scale: f32) -> [[f32; 4]; 4] {
    let c = angle.cos() * scale;
    let s = angle.sin() * scale;
    let [dx, dy, dz] = translate;
    [
        [c, 0.0, s, 0.0],
        [0.0, scale, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [dx, dy, dz, 1.0],
    ]
}

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
        filename: String,
        inner: resources::ResourceError,
    },
    CannotDetermineShaderTypeForResource {
        filename: String,
    },
    UnsupportedShaderType {
        filename: String,
        unsupported_type: String,
    },
    CompileError {
        filename: String,
        message: String,
    },
    LinkError {
        filename: String,
        message: String,
    },
    SpirvReadError {
        filename: String,
        inner: std::io::Error,
    },
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderError::ResourceLoadError { filename, inner } => {
                write!(f, "{:?} ({}): {:?}", self, filename, inner)
            }
            ShaderError::CannotDetermineShaderTypeForResource { filename } => {
                write!(f, "{:?} ({})", self, filename)
            }
            ShaderError::UnsupportedShaderType {
                filename,
                unsupported_type,
            } => write!(f, "{:?} ({}): {}", self, filename, unsupported_type),
            ShaderError::CompileError { filename, message } => {
                write!(f, "{:?} ({}): {}", self, filename, message)
            }
            ShaderError::LinkError { filename, message } => {
                write!(f, "{:?} ({}): {}", self, filename, message)
            }
            ShaderError::SpirvReadError { filename, inner } => {
                write!(f, "{:?} ({}): {:?}", self, filename, inner)
            }
        }
    }
}
impl std::error::Error for ShaderError {}

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
                filename: filename.into(),
            })?;
        let bytecode =
            res.load_to_bytes(filename, false)
                .map_err(|e| ShaderError::ResourceLoadError {
                    filename: filename.into(),
                    inner: e,
                })?;
        let spirv = gfx_hal::pso::read_spirv(std::io::Cursor::new(&bytecode)).map_err(|e| {
            ShaderError::SpirvReadError {
                filename: filename.into(),
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
                filename: self.filename.clone(),
                message: format!("{:?}", e),
            })
    }
}
