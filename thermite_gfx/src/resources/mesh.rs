use crate::primitives::vertex::Vertex;
use bincode;
use thermite_core::tools::resources::{Resource, ResourceError};

/// A 3D mesh
pub struct Mesh {
    pub(crate) vertex_count: usize,
    pub(crate) vertex_data: Vec<Vertex>,
}

impl Mesh {
    /// Loads a new 3D `Mesh` located at the given `Resource`, named `filename`
    pub fn new(res: &Resource, filename: &str) -> Result<Self, ResourceError> {
        let binary_data = res.load_to_bytes(filename, false)?;
        let vertex_data: Vec<Vertex> = bincode::deserialize(&binary_data)
            .map_err(|_| ResourceError::DeserializationFailure(filename.to_string()))?;
        let vertex_count = vertex_data.len();
        Ok(Mesh {
            vertex_count: vertex_count,
            vertex_data: vertex_data,
        })
    }
}
