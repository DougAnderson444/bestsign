//! A simple blockstore resolver that fetches data from an in-memory blockstore.

use blockstore::{Blockstore as _, InMemoryBlockstore};
use cid::Cid;
use provenance_log::multicid;
use std::sync::Arc;
use std::{future::Future, pin::Pin};
use tokio::sync::Mutex;

use super::Resolver;

/// A resolver that fetches data from the blockstore.
#[derive(Clone)]
pub struct BlockstoreResolver {
    pub blockstore: Arc<Mutex<InMemoryBlockstore<64>>>,
}

impl Resolver for BlockstoreResolver {
    type Error = BlockstoreResolverError;

    fn resolve(
        &self,
        cid: &multicid::Cid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Self::Error>> + Send>> {
        let blockstore = self.blockstore.clone();
        let cid_bytes: Vec<u8> = (cid.clone()).into();
        Box::pin(async move {
            let cid = Cid::try_from(cid_bytes)?;

            let Some(block) = blockstore.lock().await.get(&cid).await? else {
                panic!("Failed to get block from blockstore");
            };
            Ok(block)
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BlockstoreResolverError {
    #[error("Blockstore error: {0}")]
    BlockstoreError(#[from] blockstore::Error),
    #[error("Cid error: {0}")]
    CidError(#[from] cid::Error),
}
