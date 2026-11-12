//! Shared progress and cancellation state between the import thread and the interface.

use crate::features::import_export::models::ImportError;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Progress (0..=1) and a cancel flag, shared through an `Arc`.
#[derive(Debug, Default)]
pub struct Progress {
    permille: AtomicU32,
    cancelled: AtomicBool,
}

impl Progress {
    /// Publishes the progress of the whole import.
    pub fn set(&self, fraction: f32) {
        self.permille.store(
            (fraction.clamp(0.0, 1.0) * 1000.0) as u32,
            Ordering::Relaxed,
        );
    }

    /// Current progress.
    pub fn get(&self) -> f32 {
        self.permille.load(Ordering::Relaxed) as f32 / 1000.0
    }

    /// Asks the import to stop.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Whether a stop was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// `Err(Cancelled)` once a stop was requested; call it inside long loops.
    pub fn check(&self) -> Result<(), ImportError> {
        if self.is_cancelled() {
            Err(ImportError::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_clamped_and_cancel_sticks() {
        let progress = Progress::default();
        progress.set(1.7);
        assert_eq!(progress.get(), 1.0);
        assert!(progress.check().is_ok());
        progress.cancel();
        assert_eq!(progress.check(), Err(ImportError::Cancelled));
    }
}
