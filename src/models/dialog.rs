//! Dialogs the application can show over the current screen.

/// An error shown in the error dialog; both texts are localization keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorInfo {
    /// Localization key of the title.
    pub title: &'static str,
    /// Localization key of the explanation.
    pub message: &'static str,
    /// Technical code shown under the explanation.
    pub code: &'static str,
}

/// Every dialog and floating panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialog {
    /// Create a new scene.
    NewScene,
    /// Open a project file.
    OpenProject,
    /// Save the project.
    SaveProject,
    /// Import a model exported from Revit or another program.
    Import,
    /// Export the current view as an image.
    ExportImage,
    /// Application preferences.
    Preferences,
    /// Full keyboard shortcut reference.
    Shortcuts,
    /// About box.
    About,
    /// Notifications panel.
    Notifications,
    /// An error report.
    Error(ErrorInfo),
}

/// What a dialog asks the host to do after a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogOutcome {
    /// Keep showing the dialog.
    Keep,
    /// Close the dialog.
    Close,
    /// Replace the dialog by another one.
    Open(Dialog),
}
