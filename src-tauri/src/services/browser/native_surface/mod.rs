#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub(super) use macos::{resolve_parent, update_browser};
#[cfg(target_os = "windows")]
pub(super) use windows::{resolve_parent, update_browser};
