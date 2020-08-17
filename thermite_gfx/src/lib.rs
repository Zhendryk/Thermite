// Re-export gfx_hal to be used by clients of thermite_gfx
pub use gfx_hal;

// Re-export winit to be used by clients of thermite_gfx
pub use winit;

// thermite_gfx modules
pub mod hal;
pub mod primitives;
pub mod resources;
pub mod shaders;
pub mod window;
