//! Application identity.

/// Name and version of the application.
pub struct Application {
    name: &'static str,
    version: &'static str,
}

impl Application {
    /// Builds the application description from the crate metadata.
    pub const fn new() -> Self {
        Self {
            name: "Guanacaste Render",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    /// Product name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Semantic version.
    pub const fn version(&self) -> &'static str {
        self.version
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}
