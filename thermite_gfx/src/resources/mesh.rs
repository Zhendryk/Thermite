use crate::primitives::vertex::Vertex;
use bincode;
use thermite_core::resources::{Resource, ResourceError};

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
}
