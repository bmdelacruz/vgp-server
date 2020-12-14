#[cfg(target_os = "linux")]
mod linux_impl;

#[cfg(target_os = "linux")]
pub use linux_impl::GamePad;
