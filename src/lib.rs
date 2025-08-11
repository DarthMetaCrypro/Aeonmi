// Make the same modules available from the library crate so anything under
// src/tui/* (compiled as part of lib) can reach them via `crate::...`.
pub mod core;
pub mod cli;
pub mod commands;
pub mod config;
pub mod tui;
