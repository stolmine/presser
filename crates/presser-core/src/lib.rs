//! Core library for Presser
//!
//! This crate ties together all the other Presser crates and provides
//! the main application logic.

pub mod commands;
pub mod engine;
pub mod tasks;
pub mod ui;

pub use commands::*;
pub use engine::Engine;
