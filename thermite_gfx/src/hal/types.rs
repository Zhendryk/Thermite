use gfx_hal;

pub(crate) type ThermiteBackend = backend::Backend;
pub(crate) type ThermiteInstance = backend::Instance;
pub(crate) type ThermiteDevice = backend::Device;
pub(crate) type ThermiteRenderPass = <ThermiteBackend as gfx_hal::Backend>::RenderPass;
pub(crate) type ThermitePipelineLayout = <ThermiteBackend as gfx_hal::Backend>::PipelineLayout;
pub(crate) type ThermiteGraphicsPipeline = <ThermiteBackend as gfx_hal::Backend>::GraphicsPipeline;
pub(crate) type ThermiteSwapchainImage =
    <<ThermiteBackend as gfx_hal::Backend>::Surface as gfx_hal::window::PresentationSurface<
        ThermiteBackend,
    >>::SwapchainImage;
pub(crate) type ThermiteFramebuffer = <ThermiteBackend as gfx_hal::Backend>::Framebuffer;

/// The error type reported by this module, regarding Hardware Abstraction Layer operation errors/failures
#[derive(Debug)]
pub enum HALError {
    UnsupportedBackend,
    InitializationError(gfx_hal::window::InitError),
    CreationError(gfx_hal::window::CreationError),
    AdapterError {
        message: String,
        inner: Option<gfx_hal::device::CreationError>,
    },
    OutOfMemory(gfx_hal::device::OomOrDeviceLost),
    ShaderError(crate::shaders::shader::ShaderError),
    PipelineError(gfx_hal::pso::CreationError),
    ResourceError(thermite_core::resources::ResourceError),
    AcquireError(gfx_hal::window::AcquireError),
    GPUNotFound,
    CannotAddGPU,
}

impl From<gfx_hal::window::InitError> for HALError {
    fn from(error: gfx_hal::window::InitError) -> Self {
        HALError::InitializationError(error)
    }
}

impl From<gfx_hal::window::CreationError> for HALError {
    fn from(error: gfx_hal::window::CreationError) -> Self {
        HALError::CreationError(error)
    }
}

impl From<gfx_hal::device::OomOrDeviceLost> for HALError {
    fn from(error: gfx_hal::device::OomOrDeviceLost) -> Self {
        HALError::OutOfMemory(error)
    }
}

impl From<gfx_hal::device::OutOfMemory> for HALError {
    fn from(error: gfx_hal::device::OutOfMemory) -> Self {
        HALError::OutOfMemory(error.into())
    }
}

impl From<crate::shaders::shader::ShaderError> for HALError {
    fn from(error: crate::shaders::shader::ShaderError) -> Self {
        HALError::ShaderError(error)
    }
}

impl From<gfx_hal::pso::CreationError> for HALError {
    fn from(error: gfx_hal::pso::CreationError) -> Self {
        HALError::PipelineError(error)
    }
}

impl From<thermite_core::resources::ResourceError> for HALError {
    fn from(error: thermite_core::resources::ResourceError) -> Self {
        HALError::ResourceError(error)
    }
}

impl From<gfx_hal::window::AcquireError> for HALError {
    fn from(error: gfx_hal::window::AcquireError) -> Self {
        HALError::AcquireError(error)
    }
}

impl std::fmt::Display for HALError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HALError::UnsupportedBackend => write!(fmt, "{:?}", self),
            HALError::InitializationError(err) => write!(fmt, "{:?}: {}", self, err),
            HALError::CreationError(err) => write!(fmt, "{:?}: {}", self, err),
            HALError::AdapterError { message, inner } => {
                write!(fmt, "{:?}: {} => {:?}", self, message, inner)
            }
            HALError::OutOfMemory(err) => write!(fmt, "{:?}: {}", self, err),
            HALError::ShaderError(err) => write!(fmt, "{:?}: {}", self, err),
            HALError::PipelineError(err) => write!(fmt, "{:?}: {}", self, err),
            HALError::ResourceError(err) => write!(fmt, "{:?}: {}", self, err),
            HALError::AcquireError(err) => write!(fmt, "{:?}: {}", self, err),
            HALError::GPUNotFound => write!(fmt, "{:?}", self),
            HALError::CannotAddGPU => write!(fmt, "{:?}", self),
        }
    }
}

impl std::error::Error for HALError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HALError::InitializationError(err) => Some(err),
            HALError::CreationError(err) => Some(err),
            HALError::OutOfMemory(err) => Some(err),
            HALError::ShaderError(err) => Some(err),
            HALError::PipelineError(err) => Some(err),
            HALError::ResourceError(err) => Some(err),
            HALError::AcquireError(err) => Some(err),
            _ => None,
        }
    }
}
