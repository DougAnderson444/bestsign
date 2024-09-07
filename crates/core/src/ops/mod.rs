/// Create a new provenance log
pub mod create;
pub use create::create;

/// Update a provenance log
pub mod update;

pub mod config;

mod traits;
use traits::{EntrySigner, KeyManager};

/// Handy export for all public symbols
pub mod prelude {
    pub use super::*;
}
