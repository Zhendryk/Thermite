use bincode;
use gfx_hal::{
    self,
    adapter::PhysicalDevice,
    buffer::Usage,
    device::Device,
    memory::{Properties, Segment},
    MemoryTypeId,
};
use serde;
use thermite_core::resources::{Resource, ResourceError};

#[repr(C)]
#[derive(serde::Deserialize)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

/// A 3D mesh
pub struct Mesh {
    pub(crate) vertex_count: usize,
    // pub(crate) binary_data: Vec<u8>,
    pub(crate) vertex_data: Vec<Vertex>,
}

impl Mesh {
    pub fn new(res: &Resource, filename: &str) -> Result<Self, ResourceError> {
        let binary_data = res.load_to_bytes(filename, false)?;
        let vertex_data: Vec<Vertex> = bincode::deserialize(&binary_data)
            .map_err(|e| ResourceError::DeserializationFailure)?;
        let vertex_count = vertex_data.len();
        Ok(Mesh {
            vertex_count: vertex_count,
            // binary_data: binary_data,
            vertex_data: vertex_data,
        })
    }

    pub fn vertex_buffer<B: gfx_hal::Backend>(
        &self,
        logical_device: &B::Device,
        physical_device: &B::PhysicalDevice,
    ) -> (B::Memory, B::Buffer) {
        let vertex_buffer_len = self.vertex_count * std::mem::size_of::<Vertex>();
        let (vertex_buffer_memory, vertex_buffer) = unsafe {
            make_buffer::<B>(
                logical_device,
                physical_device,
                vertex_buffer_len,
                Usage::VERTEX,
                Properties::CPU_VISIBLE,
            )
        };
        unsafe {
            let mapped_memory = logical_device
                .map_memory(&vertex_buffer_memory, Segment::ALL)
                .expect("TODO");
            std::ptr::copy_nonoverlapping(
                self.vertex_data.as_ptr() as *const u8,
                mapped_memory,
                vertex_buffer_len,
            );
            logical_device
                .flush_mapped_memory_ranges(vec![(&vertex_buffer_memory, Segment::ALL)])
                .expect("TODO");
            logical_device.unmap_memory(&vertex_buffer_memory);
        };
        (vertex_buffer_memory, vertex_buffer)
    }
}

/// Create a memory buffer of the specified `buffer_len`, of type `usage`
unsafe fn make_buffer<B: gfx_hal::Backend>(
    logical_device: &B::Device,
    physical_device: &B::PhysicalDevice,
    buffer_len: usize,
    usage: gfx_hal::buffer::Usage,
    properties: gfx_hal::memory::Properties,
) -> (B::Memory, B::Buffer) {
    // Create a buffer object
    let mut buffer = logical_device
        .create_buffer(buffer_len as u64, usage)
        .expect("Failed to create vertex buffer");
    // Get the requirements our logical device places on our buffer
    let req = logical_device.get_buffer_requirements(&buffer);
    //Find the correct memory type for our requirements
    let memory_types = physical_device.memory_properties().memory_types;
    let memory_type = memory_types
        .iter()
        .enumerate()
        .find(|(id, mem_type)| {
            let type_supported = req.type_mask & (1_u64 << id) != 0;
            type_supported && mem_type.properties.contains(properties)
        })
        .map(|(id, _ty)| MemoryTypeId(id))
        .expect("No compatible memory type available");
    // Allocate enough memory to fit our `buffer_len` requirement and bind it to the buffer object
    let buffer_memory = logical_device
        .allocate_memory(memory_type, req.size)
        .expect("Failed to allocate vertex buffer memory");
    logical_device
        .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
        .expect("Failed to bind vertex buffer memory");
    (buffer_memory, buffer)
}
