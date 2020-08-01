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

#[derive(Debug)] // TODO: Display, others...
pub enum BufferError {
    CreationError {
        inner: gfx_hal::buffer::CreationError,
    },
    NoCompatibleMemoryType,
    AllocationFailure {
        inner: gfx_hal::device::AllocationError,
    },
    BindFailure {
        inner: gfx_hal::device::BindError,
    },
    OutOfMemory {
        inner: gfx_hal::device::OutOfMemory,
    },
    MappingError {
        inner: gfx_hal::device::MapError,
    },
}

// TODO: Really dig into gfx_hal::Backend::Buffer/Memory to make this class robust
pub struct Buffer<B: Backend> {
    pub(crate) memory: B::Memory,
    pub(crate) buffer: B::Buffer,
}

impl<B: Backend> Buffer<B> {
    pub unsafe fn new(
        logical_device: &B::Device,
        physical_device: &B::PhysicalDevice,
        size: usize,
        usage: Usage,
        properties: Properties,
    ) -> Result<Self, BufferError> {
        // Create a buffer object
        let mut buffer = logical_device
            .create_buffer(size as u64, usage)
            .map_err(|e| BufferError::CreationError { inner: e })?;
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
        let buffer_memory = logical_device
            .allocate_memory(memory_type, req.size)
            .map_err(|e| BufferError::AllocationFailure { inner: e })?;
        logical_device
            .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
            .map_err(|e| BufferError::BindFailure { inner: e })?;
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
            let mapped_memory = logical_device
                .map_memory(&memory_buffer.memory, Segment::ALL) // NOTE: We can do this because we made the memory CPU visible... might not be the case if we don't
                .map_err(|e| BufferError::MappingError { inner: e })?;
            // Copy our vertex data into the mapped memory region
            // NOTE: This region should not overlap with where the vertices are currently allocated
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const u8,
                mapped_memory,
                buffer_size,
            );
            // Flush our mapped memory range to the GPU
            logical_device
                .flush_mapped_memory_ranges(vec![(&memory_buffer.memory, Segment::ALL)])
                .map_err(|e| BufferError::OutOfMemory { inner: e })?;
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
