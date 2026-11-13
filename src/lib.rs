//! Guanacaste Render: desktop interface for real-time architectural visualization.

// The lint wants `as_chunks`, which is not stable on the toolchains this project supports.
#![allow(clippy::chunks_exact_to_as_chunks)]

pub mod commands;
pub mod components;
pub mod config;
pub mod core;
pub mod features;
pub mod models;
pub mod screens;
pub mod services;
pub mod utils;

pub use core::run;
