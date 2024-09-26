/// Create a new provenance log
pub mod open;
pub use open::create;

/// Update a provenance log
pub mod update;
pub use update::update_plog;

pub mod config;

mod traits;
pub use traits::CryptoManager;

/// Handy export for all public symbols
pub mod prelude {
    pub use super::*;
}
