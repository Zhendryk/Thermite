// Re-export simple_logger as thermite_logging to be used by clients of thermite_core
// TODO: Explore if simple_logger is actually performant enough for this project.
pub use simple_logger as thermite_logging;

// thermite_core native modules
pub mod input;
pub mod platform;
pub mod tools;
