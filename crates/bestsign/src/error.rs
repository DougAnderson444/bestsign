/// Errors generated from this crate
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Open operation errors
    #[error(transparent)]
    Open(#[from] OpenError),
    /// Update operation errors
    #[error(transparent)]
    Update(#[from] UpdateError),

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
    /// Invalid OpParams
    #[error("Invalid op params")]
    InvalidOpParams,
}

/// Update op errors
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UpdateError {
    /// Invalid CID params
    #[error("Invalid cid params")]
    InvalidCidParams,
    /// No op key-path
    #[error("Missing op key-path")]
    NoOpKeyPath,
    /// No update op value
    #[error("Missing update op value")]
    NoUpdateOpValue,
    /// Invalid OpParams
    #[error("Invalid op params")]
    InvalidOpParams,
}
