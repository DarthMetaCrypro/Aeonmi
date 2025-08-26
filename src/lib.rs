// Make the same modules available from the library crate so anything under
// src/tui/* (compiled as part of lib) can reach them via `crate::...`.
pub mod cli;
pub mod commands;
pub mod config;
pub mod core;
pub mod io;
pub mod tui;
// Optional: expose GUI bridge commands if building with that feature
#[cfg(any())]
pub mod gui; // placeholder if gui modules structured under src/gui

// Re-export tauri bridge commands if the path exists (using conditional to avoid compile fail if not included)
#[allow(unused_imports)]
pub use crate::commands::*; // existing commands
