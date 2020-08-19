// Re-export gfx_hal to be used by clients of thermite_gfx
pub use gfx_hal;

// Re-export winit to be used by clients of thermite_gfx
pub use winit;

// thermite_gfx native modules
pub mod hal;
pub mod primitives;
pub mod resources;
pub mod shaders;
pub mod window;

// use std::boxed::Box;
// use thermite_core::platform::layer::Layer;
// pub struct GraphicsLayer {
//     hal_state: Box<hal::hal_state::HALState>,
//     window: Box<window::Window>,
// }

// impl Default for GraphicsLayer {
//     fn default() -> Self {
//         let window = window::Window::default();
//         let hal_state =
//             hal::hal_state::HALState::new(window.handle()).expect("Couldn't create HALState");
//         GraphicsLayer {
//             hal_state: Box::new(hal_state),
//             window: Box::new(window),
//         }
//     }
// }

// impl Layer for GraphicsLayer {
//     fn on_attach(&self) {}

//     fn on_detach(&self) {}

//     fn on_update(&self) {}

//     fn identifier(&self) -> u32 {
//         0
//     }

//     fn debug_name(&self) -> &str {
//         "Graphics Layer"
//     }
// }
