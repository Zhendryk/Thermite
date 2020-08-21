// Re-export simple_logger as thermite_logging to be used by clients of thermite_core
pub use simple_logger as thermite_logging;

// thermite_core native modules
pub mod input;
pub mod messaging;
pub mod platform;
pub mod tools;
