//! When a consumer gets a verifiable long-lived address (VLAD), it maps
//! to the Plog Head Content Identifier (CID) that maps to the actual data stored
//! somewhere in content addressed storage.
//!
//! The Data Provenance Log (plog) needs the thus resolve the CID to the
//! actual plog entries at which the CIDs point.
//!
//! This process is called resolving, and since the content addressed storage
//! could be many types, the resolution process is defined by the user.
//!
//! Users can implement the [Resolver] trait to define how to resolve the data
//! from a CID chain. Then, the [get_entry_chain] function can be used to get
//! the entries from the head CID down to the foot CID.

#[cfg(feature = "blockstore")]
pub mod blockstore_resolver;

use provenance_log::{multicid, multicodec, multihash, multitrait, multiutil};

use indexmap::IndexMap;
use multicid::Cid;
use multitrait::Null;
use multiutil::CodecInfo;
use provenance_log::Entry;
use std::{cmp::Ordering, future::Future, pin::Pin};

/// Error types for resolution operations
#[derive(thiserror::Error, Debug)]
pub enum ResolveError {
    #[error("Failed to get block from blockstore")]
    BlockNotFound,

    #[error("Log verification failed: {0}")]
    VerificationError(String),

    #[error("CID mismatch: expected {expected}, got {actual}")]
    CidMismatch {
        expected: multicid::Cid,
        actual: multicid::Cid,
    },

    #[error("Failed to get last entry")]
    NoLastEntry,

    #[error("Other error: {0}")]
    Other(#[from] Box<dyn std::error::Error>),
}

/// A trait for resolving data from a Cid.
///
/// # Example
///
/// ```rust
/// use std::pin::Pin;
/// use std::future::Future;
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
/// use bestsign_core::{Entry, Cid};
/// use blockstore::{Blockstore as _, InMemoryBlockstore};
/// use bestsign_core::resolve::Resolver;
///
/// struct Resolve {
///    pub blockstore: Arc<Mutex<InMemoryBlockstore<64>>>,
/// }
///
/// impl Resolver for Resolve {
///    type Error = TestError;
///
///    fn resolve(
///        &self,
///        cid: &Cid,
///    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Self::Error>> + Send>> {
///        let blockstore = self.blockstore.clone();
///        let cid_bytes: Vec<u8> = (cid.clone()).into();
///        Box::pin(async move {
///            let cid = cid::Cid::try_from(cid_bytes)?;
///            let cid = cid::Cid::try_from(cid_bytes)?;
///
///            let block = blockstore.lock().await.get(&cid).await?
///                .ok_or(TestError::BlockstoreError("Failed to get block from blockstore".into()))?;
///            Ok(block)
///        })
///    }
/// }
///
/// #[derive(thiserror::Error, Debug)]
/// enum TestError {
///    #[error("Blockstore error: {0}")]
///    BlockstoreError(#[from] blockstore::Error),
///    #[error("Cid error: {0}")]
///    CidError(#[from] cid::Error),
/// }
///```
#[allow(clippy::type_complexity)]
pub trait Resolver {
    type Error: std::error::Error + 'static;

    fn resolve(
        &self,
        cid: &Cid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Self::Error>> + Send>>;
}

/// Recursively get the resolved data from a head [Cid] down to the foot [Cid],
/// returning the list of [Entry]s for the [Log]. Uses the [Resolver] to fetch
/// the data.
///
/// The returned [IndexMap] retains order, so the entries are in order from head to foot.
pub async fn get_entry_chain(
    head_cid: Cid,
    get_data: impl Resolver,
) -> Result<IndexMap<multicid::Cid, Entry>, ResolveError> {
    let mut entries = IndexMap::new();
    let mut current_cid = head_cid;
    loop {
        let entry_bytes = get_data
            .resolve(&current_cid)
            .await
            .map_err(|e| ResolveError::Other(Box::new(e)))?;

        // entry bytes Cid should match the given Cid
        let rebuilt_cid = multicid::cid::Builder::new(multicodec::Codec::Cidv1)
            .with_target_codec(current_cid.target_codec)
            .with_hash(
                &multihash::Builder::new_from_bytes(current_cid.hash.codec(), &entry_bytes)
                    .map_err(|e| ResolveError::Other(Box::new(e)))?
                    .try_build()
                    .map_err(|e| ResolveError::Other(Box::new(e)))?,
            )
            .try_build()
            .map_err(|e| ResolveError::Other(Box::new(e)))?;

        // error if the Cids don't match
        if rebuilt_cid != current_cid {
            return Err(ResolveError::CidMismatch {
                expected: current_cid,
                actual: rebuilt_cid,
            });
        }

        let entry = Entry::try_from(entry_bytes.as_slice())
            .map_err(|e| ResolveError::Other(Box::new(e)))?;
        entries.insert(current_cid.clone(), entry.clone());

        // We stop when we reach the foot, which has a Null prev
        if entry.prev() == Null::null() {
            break;
        }
        current_cid = entry.prev();
    }

    Ok(entries)
}

/// Result of resolving and verifying a Plog
#[derive(Debug, Clone)]
pub struct ResolvedPlog {
    /// The reconstructed and verified provenance log
    pub log: provenance_log::Log,

    /// The verification counts for each step in the verification process
    /// Stored in order from latest to earliest entry
    pub verification_counts: Vec<usize>,
}

impl ResolvedPlog {
    /// Compare this resolved plog with another and determine which has the lower verification cost
    ///
    /// Returns:
    /// - Ordering::Less if this plog has a lower verification cost
    /// - Ordering::Greater if the other plog has a lower verification cost
    /// - Ordering::Equal if they have the same verification cost
    ///
    /// The comparison is done by examining each verification step, starting from the most recent entry.
    /// At the first differing count, the plog with the lower count is considered "less".
    pub fn compare(&self, other: &ResolvedPlog) -> std::cmp::Ordering {
        // Compare counts from the most recent (latest) entries first
        for (self_count, other_count) in self
            .verification_counts
            .iter()
            .zip(other.verification_counts.iter())
        {
            match self_count.cmp(other_count) {
                std::cmp::Ordering::Equal => continue, // If equal, check the next count
                order => return order, // Return the ordering at the first difference
            }
        }

        // If we've compared all common entries and they're equal, compare by length
        // Longer chains are kept as this is how they are updated
        // Keep longer is same.
        other
            .verification_counts
            .len()
            .cmp(&self.verification_counts.len())
    }

    /// Calculate the total verification count across all steps
    pub fn total_count(&self) -> usize {
        self.verification_counts.iter().sum()
    }

    /// Returns true if this plog has a lower verification cost than the other
    pub fn is_cheaper_than(&self, other: &ResolvedPlog) -> bool {
        self.compare(other) == std::cmp::Ordering::Less
    }
}

/// Given the vlad and the head Cid, resolve the Plog entries,
/// rebuild the Plog, and verify it.
pub async fn resolve_plog(
    vlad: &multicid::Vlad,
    head_cid: &multicid::Cid,
    resolver: impl Resolver + Clone,
) -> Result<ResolvedPlog, ResolveError> {
    let fetched_entries = get_entry_chain(head_cid.clone(), resolver.clone()).await?;

    // Reconstruct the plog from the fetched entries
    let first_lock_cid = vlad.cid();

    let entry_bytes = resolver
        .resolve(first_lock_cid)
        .await
        .map_err(|e| ResolveError::Other(Box::new(e)))?;

    let maybe_first_lock_script = provenance_log::Script::try_from(entry_bytes.as_slice())
        .map_err(|e| ResolveError::Other(Box::new(e)))?;

    // Get the last entry for the foot
    let last_entry = fetched_entries.last().ok_or(ResolveError::NoLastEntry)?;

    let rebuilt_plog = provenance_log::log::Builder::new()
        // we'll get this from the DHT record key
        .with_vlad(vlad)
        // First lock script CID is the second half of the vlad
        .with_first_lock(&maybe_first_lock_script)
        // we get these entries from the network
        .with_entries(&fetched_entries.clone().into_iter().collect())
        // We will have the head from the DHT record value
        .with_head(head_cid)
        // foot is the last entry we fetched from the blockstore
        .with_foot(last_entry.0)
        .try_build()
        .map_err(|e| ResolveError::Other(Box::new(e)))?;

    let plog_clone = rebuilt_plog.clone();

    let verify_iter = &mut plog_clone.verify();

    // Check that first entry matches (using debug_assert for development checks)
    if let Some(head_entry) = fetched_entries.get(head_cid) {
        debug_assert_eq!(rebuilt_plog.entries[head_cid], head_entry.clone());
    }

    // Collect individual verification counts
    let mut verification_counts = Vec::new();

    // the log should also verify
    for ret in verify_iter {
        match ret {
            Ok((count, entry, kvp)) => {
                verification_counts.push(count);
                tracing::trace!("Verified entry: {:#?}", entry);
                tracing::trace!("Verified count: {:#?}", count);
                tracing::trace!("Verified kvp: {:#?}", kvp);
            }
            Err(e) => {
                tracing::error!("Error: {:#?}", e);
                return Err(ResolveError::VerificationError(e.to_string()));
            }
        }
    }

    Ok(ResolvedPlog {
        log: rebuilt_plog,
        verification_counts,
    })
}

#[cfg(test)]
mod tests {
    use crate::blockstore_resolver::BlockstoreResolver;

    use super::*;
    use blockstore::{Blockstore as _, InMemoryBlockstore};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_resolve_plog() {
        let blockstore = Arc::new(Mutex::new(InMemoryBlockstore::<64>::new()));
        let resolver = BlockstoreResolver { blockstore };
    }
}
