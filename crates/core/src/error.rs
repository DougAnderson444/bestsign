use provenance_log::{multicid, multihash, multikey, multiutil};

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

    /// Plog operation errors
    #[error(transparent)]
    Plog(#[from] PlogError),

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
    /// Generic Error
    #[error("Error: {0}")]
    Generic(String),

    /// Utility conversion error
    #[error(transparent)]
    MultiUtil(#[from] multiutil::Error),
}

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
    /// No last entry
    #[error("No last entry")]
    NoLastEntry,
}

/// Plog errors
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PlogError {
    /// An incorrect key-path
    #[error("Invalid key-path")]
    InvalidKeyPath,
    /// Invalid file params
    #[error("Invalid file params")]
    InvalidFileParams,
    /// Invalid key params
    #[error("Invalid key params")]
    InvalidKeyParams,
    /// An incorrect value type was encountered
    #[error("Invalid WACC value type")]
    InvalidVMValue,
    /// No p.log command
    #[error("No p.log command")]
    NoCommand,
    /// No first entry
    #[error("P.log missing first entry")]
    NoFirstEntry,
    /// No vlad key in the first entry
    #[error("P.log missing VLAD verification key in first entry")]
    NoVladKey,
    /// No input file given
    #[error("No input file given")]
    NoInputFile,
    /// No key-path specified
    #[error("No key-path specified")]
    NoKeyPath,
    /// No codec specified
    #[error("No codec specified")]
    NoCodec,
    /// No string value given
    #[error("No string value given")]
    NoStringValue,
}

impl From<Error> for multicid::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Multicid(e) => e,
            _ => multicid::Error::Multikey(multikey::Error::Sign(
                multikey::error::SignError::SigningFailed(e.to_string()),
            )),
        }
    }
}
