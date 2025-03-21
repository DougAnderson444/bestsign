/// Error Module
pub mod error;
pub use error::Error;

/// Operations
pub mod ops;
pub use ops::open::Codec;

/// Export provenance log utils
pub use provenance_log;
pub use provenance_log::{multicid, multikey, multisig, multitrait};

pub use multicid::{Cid, EncodedVlad, Vlad};
pub use multikey::multibase::Base;
pub use multikey::{mk, EncodedMultikey, Multikey, Views};
pub use multisig::Multisig;

pub use multitrait::Null;

pub mod utils;

/// Resolving utilities
pub mod resolve;

/// If [blockstore] feature is enabled, export blockstore module
#[cfg(feature = "blockstore")]
pub use resolve::blockstore_resolver;

pub use multikey::multicrates::*;
