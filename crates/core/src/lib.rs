#![feature(const_trait_impl)]

/// Error
pub mod error;
pub use error::Error;

/// Operations
pub mod ops;

/// Export provenance log Script
pub use provenance_log::{Key, Script};

mod utils;
