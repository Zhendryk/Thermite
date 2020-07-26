// use crate::thermite_core::resources;
// use gfx_hal;

// #[derive(Debug, Copy, Clone)]
// pub enum ShaderType {
//     SPIRV,
//     GLSL_VERT,
//     GLSL_FRAG,
// }

// // Shader types
// const SHADER_EXT: [(&str, ShaderType); 1] = [(".spv", ShaderType::SPIRV)];

// // Errors relating to `Shader`s and `ShaderProgram`s
// // TODO: impl Display for these
// #[derive(Debug)]
// pub enum ShaderError {
//     ResourceLoadError {
//         name: String,
//         inner: resources::ResourceError,
//     },
//     CannotDetermineShaderTypeForResource {
//         name: String,
//     },
//     UnsupportedShaderType {
//         name: String,
//     },
//     CompileError {
//         name: String,
//         message: String,
//     },
//     LinkError {
//         name: String,
//         message: String,
//     },
// }

// pub struct Shader {
//     pub kind: ShaderType,
//     pub id: Option<u32>,
//     pub data: Option<Vec<u32>>,
// }

// impl Shader {
//     pub fn new(res: &resources::Resource, filename: &str) -> Result<Shader, ShaderError> {
//         let shader_type = SHADER_EXT
//             .iter()
//             .find(|&&(ext, _)| filename.ends_with(ext))
//             .map(|&(_, kind)| kind)
//             .ok_or_else(|| ShaderError::CannotDetermineShaderTypeForResource {
//                 name: filename.into(),
//             })?;
//         match shader_type {
//             ShaderType::SPIRV => {
//                 let spirv_bytes =
//                     res.load_to_bytes(filename)
//                         .map_err(|e| ShaderError::ResourceLoadError {
//                             name: filename.into(),
//                             inner: e,
//                         })?;
//                 let spirv = gfx_hal::pso::read_spirv(std::io::Cursor::new(&spirv_bytes))
//                     .expect("Invalid SPIR-V shader");
//                 Ok(Shader {
//                     kind: shader_type,
//                     id: Option::None,
//                     data: Option::from(spirv),
//                 })
//             }
//             _ => Err(ShaderError::UnsupportedShaderType {
//                 name: filename.into(),
//             }),
//         }
//     }
// }
