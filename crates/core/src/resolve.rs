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
use provenance_log::{multicid, multicodec, multihash, multitrait, multiutil};

use indexmap::IndexMap;
use multicid::Cid;
use multitrait::Null;
use multiutil::CodecInfo;
use provenance_log::Entry;
use std::{future::Future, pin::Pin};

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
///
///            let Some(block) = blockstore.lock().await.get(&cid).await? else {
///                panic!("Failed to get block from blockstore");
///            };
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
) -> Result<IndexMap<multicid::Cid, Entry>, Box<dyn std::error::Error>> {
    let mut entries = IndexMap::new();
    let mut current_cid = head_cid;
    loop {
        let entry_bytes = get_data.resolve(&current_cid).await?;

        // entry bytes Cid should match the given Cid
        let rebuilt_cid = multicid::cid::Builder::new(multicodec::Codec::Cidv1)
            .with_target_codec(current_cid.target_codec)
            .with_hash(
                &multihash::Builder::new_from_bytes(current_cid.hash.codec(), &entry_bytes)
                    .unwrap()
                    .try_build()
                    .unwrap(),
            )
            .try_build()
            .unwrap();

        // error if the Cids don't match
        if rebuilt_cid != current_cid {
            return Err("Cid mismatch".into());
        }

        let entry = Entry::try_from(entry_bytes.as_slice())?;
        entries.insert(current_cid.clone(), entry.clone());

        // We stop when we reach the foot, which has a Null prev
        if entry.prev() == Null::null() {
            break;
        }
        current_cid = entry.prev();
    }

    Ok(entries)
}
