#[cfg(target_os = "linux")]
pub mod linux;
pub mod macos;
pub mod nvidia;
pub mod tracker;
pub mod windows;

pub use tracker::{detect_hardware, resolve_target_acceleration, GpuTracker};
