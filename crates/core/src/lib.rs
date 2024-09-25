/// Error Module
pub mod error;
pub use error::Error;

/// Operations
pub mod ops;
pub use ops::open::Codec;

pub use multikey::Multikey;
pub use multisig::Multisig;

/// Export provenance log Script
pub use provenance_log::{Key, Log, Script};

mod utils;
