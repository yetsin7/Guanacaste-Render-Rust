//! Guanacaste Render: desktop interface for real-time architectural visualization.

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
