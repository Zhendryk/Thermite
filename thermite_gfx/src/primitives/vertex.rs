use serde::Deserialize;

#[repr(C)]
#[derive(Deserialize)]
// TODO: Abstract this to where you can pass in the dimensionality
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}
