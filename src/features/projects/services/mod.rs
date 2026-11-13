//! Persistence of projects: the `.gr` container (`GRRPROJ1`), atomic saving, loading with limits, recovery copies and the
//! recent-projects list.

pub mod container;
pub mod open;
pub mod recent;
pub mod recovery;
pub mod store;
pub mod util;
