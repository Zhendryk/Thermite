use gfx_hal::{
    self,
    pso::{ShaderStageFlags, Specialization},
};

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

/// Errors returned by this module, related to operations with `Shader`s
#[derive(Debug)]
pub enum ShaderError {
    ResourceLoadError(thermite_core::resources::ResourceError),
    CannotDetermineShaderTypeForResource(String),
    UnsupportedShaderType(String),
    CompileError(gfx_hal::device::ShaderError),
    SpirvReadError {
        filename: String,
        inner: std::io::Error,
    },
    VertexShaderRequired,
    ShaderModuleNotCompiled,
}

impl From<thermite_core::resources::ResourceError> for ShaderError {
    fn from(error: thermite_core::resources::ResourceError) -> Self {
        ShaderError::ResourceLoadError(error)
    }
}

impl From<gfx_hal::device::ShaderError> for ShaderError {
    fn from(error: gfx_hal::device::ShaderError) -> Self {
        ShaderError::CompileError(error)
    }
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderError::ResourceLoadError(error) => write!(fmt, "{:?}: {}", self, error),
            ShaderError::CannotDetermineShaderTypeForResource(filename) => {
                write!(fmt, "{:?}: {}", self, filename)
            }
            ShaderError::UnsupportedShaderType(filename) => write!(fmt, "{:?}: {}", self, filename),
            ShaderError::CompileError(err) => write!(fmt, "{:?}: {}", self, err),
            ShaderError::SpirvReadError { filename, inner } => {
                write!(fmt, "{:?} ({}): {}", self, filename, inner)
            }
            ShaderError::VertexShaderRequired => write!(fmt, "{:?}", self),
            ShaderError::ShaderModuleNotCompiled => write!(fmt, "{:?}: Attempted an operation that requires a compiled shader module before it existed.", self)
        }
    }
}

impl std::error::Error for ShaderError {}

/// Structure containing all of the information needed to create and use a Shader in a rendering pipeline
pub struct Shader<B: gfx_hal::Backend> {
    filename: String,
    stage: gfx_hal::pso::ShaderStageFlags,
    entry: String,
    spirv: Vec<u32>,
    specialization: gfx_hal::pso::Specialization<'static>,
    module: Option<B::ShaderModule>,
}

impl<B: gfx_hal::Backend> Shader<B> {
    /// Create a new `Shader` of type `stage` with the given `entry` and `specialization` residing inside the given `Resource`
    pub fn new(
        res: &thermite_core::resources::Resource,
        filename: &str,
        stage: gfx_hal::pso::ShaderStageFlags,
        entry: &str,
        specialization: gfx_hal::pso::Specialization<'static>,
    ) -> Result<Shader<B>, ShaderError> {
        let bytecode = res.load_to_bytes(filename, false)?;
        let spirv = gfx_hal::pso::read_spirv(std::io::Cursor::new(&bytecode)).map_err(|e| {
            ShaderError::SpirvReadError {
                filename: filename.to_string(),
                inner: e,
            }
        })?;
        Ok(Shader {
            filename: filename.to_string(),
            stage: stage,
            entry: entry.to_string(),
            spirv: spirv,
            specialization: specialization,
            module: None,
        })
    }

    /// Interally compile and store this `Shader`'s module
    pub(crate) unsafe fn compile_module(
        &mut self,
        logical_device: &B::Device,
    ) -> Result<(), ShaderError> {
        use gfx_hal::device::Device;
        Ok(self.module = Some(logical_device.create_shader_module(&self.spirv)?))
    }

    /// Generate and return this `Shader`'s `EntryPoint` to be used in a `ShaderSet`
    pub(crate) fn entrypoint<'a>(&'a self) -> Result<gfx_hal::pso::EntryPoint<'a, B>, ShaderError> {
        Ok(gfx_hal::pso::EntryPoint {
            entry: &self.entry,
            module: self
                .module
                .as_ref()
                .ok_or(ShaderError::ShaderModuleNotCompiled)?,
            specialization: self.specialization.clone(),
        })
    }

    /// Free the memory associated with this `Shader`'s module
    pub fn destroy(&mut self, logical_device: &B::Device) {
        if let Some(module) = self.module.take() {
            use gfx_hal::device::Device;
            unsafe {
                logical_device.destroy_shader_module(module);
            }
        }
        self.module = None
    }
}

impl<B: gfx_hal::Backend> Drop for Shader<B> {
    fn drop(&mut self) {
        if self.module.is_some() {
            panic!("This shader class needs to be manually dropped with destroy() first");
        }
    }
}

use std::collections::HashMap;

/// Structure containing all of the `Shader`s to be used in a rendering pipeline, as a single set
pub struct ShaderSet<B: gfx_hal::Backend> {
    shaders: HashMap<gfx_hal::pso::ShaderStageFlags, Shader<B>>,
}

impl<'a, B: gfx_hal::Backend> ShaderSet<B> {
    /// Creates a `ShaderSet` including all shader types denoted by the `using_stages` bitfield residing at the given `Resource`
    ///
    /// **NOTE:** All shader files in a single set must be named `set_name.extension`, and have the same entrypoint: `entry`
    pub unsafe fn new(
        set_name: &str,
        res: &thermite_core::resources::Resource,
        using_stages: gfx_hal::pso::ShaderStageFlags,
        entry: &'a str, // TODO: Should this be a vec, matched in size to num of stage flags?
        logical_device: &B::Device,
    ) -> Result<Self, ShaderError> {
        if (using_stages & ShaderStageFlags::VERTEX).is_empty() {
            Err(ShaderError::VertexShaderRequired)
        } else {
            let mut shaders = HashMap::new();
            let mut vertex_shader = Shader::new(
                res,
                &format!("{}.vert.spv", set_name),
                ShaderStageFlags::VERTEX,
                &entry,
                Specialization::default(),
            )?;
            vertex_shader.compile_module(logical_device)?;
            shaders.insert(ShaderStageFlags::VERTEX, vertex_shader);
            if !(using_stages & ShaderStageFlags::HULL).is_empty() {
                if let Ok(mut hull_shader) = Shader::new(
                    res,
                    &format!("{}.hull.spv", set_name),
                    ShaderStageFlags::HULL,
                    &entry,
                    Specialization::default(),
                ) {
                    hull_shader.compile_module(logical_device)?;
                    shaders.insert(ShaderStageFlags::HULL, hull_shader);
                }
            }
            if !(using_stages & ShaderStageFlags::DOMAIN).is_empty() {
                if let Ok(mut domain_shader) = Shader::new(
                    res,
                    &format!("{}.dom.spv", set_name),
                    ShaderStageFlags::DOMAIN,
                    &entry,
                    Specialization::default(),
                ) {
                    domain_shader.compile_module(logical_device)?;
                    shaders.insert(ShaderStageFlags::DOMAIN, domain_shader);
                }
            }
            if !(using_stages & ShaderStageFlags::GEOMETRY).is_empty() {
                if let Ok(mut geometry_shader) = Shader::new(
                    res,
                    &format!("{}.geo.spv", set_name),
                    ShaderStageFlags::GEOMETRY,
                    &entry,
                    Specialization::default(),
                ) {
                    geometry_shader.compile_module(logical_device)?;
                    shaders.insert(ShaderStageFlags::GEOMETRY, geometry_shader);
                }
            }
            if !(using_stages & ShaderStageFlags::FRAGMENT).is_empty() {
                if let Ok(mut fragment_shader) = Shader::new(
                    res,
                    &format!("{}.frag.spv", set_name),
                    ShaderStageFlags::FRAGMENT,
                    &entry,
                    Specialization::default(),
                ) {
                    fragment_shader.compile_module(logical_device)?;
                    shaders.insert(ShaderStageFlags::FRAGMENT, fragment_shader);
                }
            }
            Ok(ShaderSet { shaders: shaders })
        }
    }

    /// Returns the raw `GraphicsShaderSet` structure to be used in the rendering pipeline
    pub fn inner(&'a self) -> Result<gfx_hal::pso::GraphicsShaderSet<'a, B>, ShaderError> {
        Ok(gfx_hal::pso::GraphicsShaderSet {
            vertex: self
                .shaders
                .get(&ShaderStageFlags::VERTEX)
                .ok_or(ShaderError::VertexShaderRequired)?
                .entrypoint()?,
            hull: match self.shaders.get(&ShaderStageFlags::HULL) {
                Some(hull) => Some(hull.entrypoint()?),
                None => None,
            },
            domain: match self.shaders.get(&ShaderStageFlags::DOMAIN) {
                Some(domain) => Some(domain.entrypoint()?),
                None => None,
            },
            geometry: match self.shaders.get(&ShaderStageFlags::GEOMETRY) {
                Some(geometry) => Some(geometry.entrypoint()?),
                None => None,
            },
            fragment: match self.shaders.get(&ShaderStageFlags::FRAGMENT) {
                Some(fragment) => Some(fragment.entrypoint()?),
                None => None,
            },
        })
    }

    /// Frees all shader modules associated with this `ShaderSet` and then clears this `ShaderSet`'s HashMap
    pub fn destroy(&mut self, logical_device: &B::Device) {
        for shader in self.shaders.values_mut() {
            shader.destroy(logical_device);
        }
        self.shaders.clear()
    }
}
