/// Errors generated from this crate
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Open operation errors
    #[error(transparent)]
    Open(#[from] OpenError),
    /// Multikey error
    #[error(transparent)]
    Multikey(#[from] multikey::Error),
    /// Multihash error
    #[error(transparent)]
    Multihash(#[from] multihash::Error),
    /// Multicid error
    #[error(transparent)]
    Multicid(#[from] multicid::Error),
    /// Provenance Log error
    #[error(transparent)]
    ProvenanceLog(#[from] provenance_log::Error),
}

/// Any Errors during the creation process
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OpenError {
    /// Invalid VLAD key params
    #[error("Invalid key params")]
    InvalidKeyParams,
}
