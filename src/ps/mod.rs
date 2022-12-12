#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
mod std;

#[cfg(not(windows))]
pub use std::*;