//! Application lifecycle: window, shell and start-up. No domain logic lives here.

mod app;
mod application;
mod dialogs;
#[cfg(test)]
mod qa_tests;
mod shortcuts;
pub mod window_frame;

pub use app::{run, StudioApp};
pub use application::Application;
pub use dialogs::DialogHost;
