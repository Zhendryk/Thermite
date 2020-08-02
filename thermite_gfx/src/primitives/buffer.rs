use crate::primitives::vertex::Vertex;
use crate::resources::mesh::Mesh;
use gfx_hal::{
    self,
    adapter::PhysicalDevice,
    buffer::Usage,
    device::Device,
    memory::{Properties, Segment},
    Backend, MemoryTypeId,
};

#[derive(Clone, Debug, PartialEq)]
pub enum BufferError {
    CreationError(gfx_hal::buffer::CreationError),
    NoCompatibleMemoryType,
    AllocationFailure(gfx_hal::device::AllocationError),
    BindFailure(gfx_hal::device::BindError),
    OutOfMemory(gfx_hal::device::OutOfMemory),
    MappingError(gfx_hal::device::MapError),
}

impl From<gfx_hal::buffer::CreationError> for BufferError {
    fn from(error: gfx_hal::buffer::CreationError) -> Self {
        BufferError::CreationError(error)
    }
}

impl From<gfx_hal::device::AllocationError> for BufferError {
    fn from(error: gfx_hal::device::AllocationError) -> Self {
        BufferError::AllocationFailure(error)
    }
}

impl From<gfx_hal::device::BindError> for BufferError {
    fn from(error: gfx_hal::device::BindError) -> Self {
        BufferError::BindFailure(error)
    }
}

impl From<gfx_hal::device::OutOfMemory> for BufferError {
    fn from(error: gfx_hal::device::OutOfMemory) -> Self {
        BufferError::OutOfMemory(error)
    }
}

impl From<gfx_hal::device::MapError> for BufferError {
    fn from(error: gfx_hal::device::MapError) -> Self {
        BufferError::MappingError(error)
    }
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::CreationError(err) => write!(fmt, "Failed to create buffer: {}", err),
            BufferError::NoCompatibleMemoryType => write!(
                fmt,
                "No compatible memory types available on this device for a buffer"
            ),
            BufferError::AllocationFailure(err) => {
                write!(fmt, "Failed to allocate memory for buffer: {}", err)
            }
            BufferError::BindFailure(err) => {
                write!(fmt, "Failed to bind memory to buffer: {}", err)
            }
            BufferError::OutOfMemory(err) => write!(fmt, "Out of memory: {}", err),
            BufferError::MappingError(err) => write!(fmt, "Failed to map buffer memory: {}", err),
        }
    }
}

impl std::error::Error for BufferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BufferError::CreationError(err) => Some(err),
            BufferError::AllocationFailure(err) => Some(err),
            BufferError::BindFailure(err) => Some(err),
            BufferError::OutOfMemory(err) => Some(err),
            BufferError::MappingError(err) => Some(err),
            _ => None,
        }
    }
}

// TODO: Really dig into gfx_hal::Backend::Buffer/Memory to make this class robust
pub struct Buffer<B: Backend> {
    pub(crate) memory: B::Memory,
    pub(crate) buffer: B::Buffer,
}

impl<B: Backend> Buffer<B> {
    /// NOTE: Should never be destroyed before any submitted command buffer which utilizes this buffer has finished execution.
    pub unsafe fn new(
        logical_device: &B::Device,
        physical_device: &B::PhysicalDevice,
        size: usize,
        usage: Usage,
        properties: Properties,
    ) -> Result<Self, BufferError> {
        // Create a buffer object
        let mut buffer = logical_device.create_buffer(size as u64, usage)?;
        // Get the logical device requirements for our buffer
        let req = logical_device.get_buffer_requirements(&buffer);
        // Find the correct memory type for our requirements
        let memory_types = physical_device.memory_properties().memory_types;
        let memory_type = memory_types
            .iter()
            .enumerate()
            .find(|(id, mem_type)| {
                let type_supported = req.type_mask & (1_u64 << id) != 0;
                type_supported && mem_type.properties.contains(properties)
            })
            .map(|(id, _ty)| MemoryTypeId(id))
            .ok_or(BufferError::NoCompatibleMemoryType)?;
        // Allocate enough memory to fit our `size` requirement and bind it to the buffer object
        let buffer_memory = logical_device.allocate_memory(memory_type, req.size)?;
        logical_device.bind_buffer_memory(&buffer_memory, 0, &mut buffer)?;
        Ok(Buffer {
            memory: buffer_memory,
            buffer: buffer,
        })
    }
}

pub struct VertexBuffer<B: Backend> {
    pub(crate) count: usize,
    pub(crate) data: Buffer<B>,
}

impl<B: Backend> VertexBuffer<B> {
    pub fn new(
        vertices: Vec<Vertex>,
        logical_device: &B::Device,
        physical_device: &B::PhysicalDevice,
    ) -> Result<Self, BufferError> {
        // Calculate the memory size of the buffer we need and create it
        let vertex_count = vertices.len();
        let buffer_size: usize = vertex_count * std::mem::size_of::<Vertex>();
        let memory_buffer = unsafe {
            Buffer::new(
                logical_device,
                physical_device,
                buffer_size,
                Usage::VERTEX,
                Properties::CPU_VISIBLE, // TODO: Look into passing this in instead
            )?
        };
        unsafe {
            // Map the buffer memory into application memory address space
            // NOTE: We can do this because we made the memory CPU visible... might not be the case if we don't
            let mapped_memory = logical_device.map_memory(&memory_buffer.memory, Segment::ALL)?;
            // Copy our vertex data into the mapped memory region
            // NOTE: This region should not overlap with where the vertices are currently allocated
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const u8,
                mapped_memory,
                buffer_size,
            );
            // Flush our mapped memory range to the GPU
            logical_device
                .flush_mapped_memory_ranges(vec![(&memory_buffer.memory, Segment::ALL)])?;
            // Unmap the memory now that we've flushed the vertex data from it
            logical_device.unmap_memory(&memory_buffer.memory);
        };
        Ok(VertexBuffer {
            count: vertex_count,
            data: memory_buffer,
        })
    }

    pub fn from_mesh(
        mesh: Mesh,
        logical_device: &B::Device,
        physical_device: &B::PhysicalDevice,
    ) -> Result<Self, BufferError> {
        VertexBuffer::new(mesh.vertex_data, logical_device, physical_device)
    }
}

pub struct IndexBuffer<B: Backend> {
    pub(crate) count: usize,
    pub(crate) data: Buffer<B>,
}

impl<B: Backend> IndexBuffer<B> {
    pub fn new(
        indices: Vec<u32>,
        logical_device: &B::Device,
        physical_device: &B::PhysicalDevice,
    ) -> Result<Self, BufferError> {
        // Calculate the memory size of the buffer we need and create it
        let idx_count = indices.len();
        let buffer_size: usize = idx_count * std::mem::size_of::<u32>();
        let memory_buffer = unsafe {
            Buffer::new(
                logical_device,
                physical_device,
                buffer_size,
                Usage::INDEX,
                Properties::CPU_VISIBLE, // TODO: Look into passing this in instead
            )?
        };
        unsafe {
            // Map the buffer memory into application memory address space
            // NOTE: We can do this because we made the memory CPU visible... might not be the case if we don't
            let mapped_memory = logical_device.map_memory(&memory_buffer.memory, Segment::ALL)?;
            // Copy our vertex data into the mapped memory region
            // NOTE: This region should not overlap with where the vertices are currently allocated
            std::ptr::copy_nonoverlapping(
                indices.as_ptr() as *const u8,
                mapped_memory,
                buffer_size,
            );
            // Flush our mapped memory range to the GPU
            logical_device
                .flush_mapped_memory_ranges(vec![(&memory_buffer.memory, Segment::ALL)])?;
            // Unmap the memory now that we've flushed the vertex data from it
            logical_device.unmap_memory(&memory_buffer.memory);
        };
        Ok(IndexBuffer {
            count: idx_count,
            data: memory_buffer,
        })
    }
}
