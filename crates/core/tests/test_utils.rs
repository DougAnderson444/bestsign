//! Test utilities for the core crate.
#[path = "./fixtures.rs"]
mod fixtures;
use std::{future::Future, pin::Pin, sync::Arc};

use fixtures::TestKeyManager;
use tokio::sync::Mutex;

use bestsign_core::{
    ops::{
        config::{defaults::DEFAULT_ENTRYKEY, LockScript, UnlockScript},
        create,
        open::config::NewLogBuilder,
        update::{OpParams, UpdateConfig},
        update_plog,
    },
    utils::{fetch_plog_data, Resolver},
    Key,
};
use blockstore::{Blockstore as _, InMemoryBlockstore};
use cid::Cid;

use crate::fixtures::{lock_script, unlock_script};

mod tests {

    use bestsign_core::{Entry, Script};

    use crate::fixtures::init_logger;

    use super::*;

    // A test that
    // - makes a 3 Entry Plog
    // - stores them
    // - uses the utility to fetch all the data.
    // - Checks the Plog verifies and data matches the expected data.
    #[tokio::test]
    async fn test_fetch_plog_data() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();
        let blockstore = Arc::new(Mutex::new(InMemoryBlockstore::<64>::new()));

        let lock_script = lock_script();
        let unlock_script = unlock_script();

        let config = NewLogBuilder::new(
            LockScript(lock_script.clone()),
            UnlockScript(unlock_script.clone()),
        )
        .try_build()?;

        let mut key_manager = TestKeyManager::new();

        let mut plog = create(&config, &mut key_manager)?;

        let update_cfg_first = UpdateConfig::new(
            unlock_script.clone(),
            key_manager.entry_key().clone().unwrap_or_default(),
        )
        .add_op(OpParams::Delete {
            key: Key::try_from(DEFAULT_ENTRYKEY).unwrap(),
        })
        // Entry lock scripts define conditions which must be met by the next entry in the plog for it to be valid.
        .add_lock(Key::try_from("/delegated/")?, lock_script.clone())
        .build();

        update_plog(&mut plog, &update_cfg_first, &mut key_manager)?;

        let update_cfg_second = UpdateConfig::new(
            unlock_script.clone(),
            key_manager.entry_key().clone().unwrap_or_default(),
        )
        .add_op(OpParams::UseStr {
            key: Key::try_from("/hello/")?,
            s: "World!".to_string(),
        })
        .build();

        update_plog(&mut plog, &update_cfg_second, &mut key_manager)?;

        // plog should verify
        let plog_clone = plog.clone();
        let verify_iter = &mut plog_clone.verify();

        for ret in verify_iter {
            if let Some(e) = ret.err() {
                tracing::error!("Error: {:#?}", e);
                // fail test
                panic!("Error in log verification");
            }
        }

        // add the first lock CID to the blockstore
        // first we need to convert from multicid::Cid to cid::Cid
        let first_lock_cid = plog.vlad.cid();
        let first_lock_cid_bytes: Vec<u8> = first_lock_cid.clone().into();
        let first_lock_cid = Cid::try_from(first_lock_cid_bytes)?;

        // Next we need to extract the first lock script as Script
        let first_lock_bytes: Vec<u8> = plog.first_lock.into();

        // the Cid of the extracted bytes should match those in the vlad.cid()
        //assert_eq!(first_lock_cid, plog.vlad.cid());

        blockstore
            .lock()
            .await
            .put_keyed(&first_lock_cid, &first_lock_bytes)
            .await?;

        // Put all the entries in the blockstore
        for (multi_cid, entry) in plog.entries.clone() {
            let entry_bytes: Vec<u8> = entry.into();

            // we need to convert multicid::Cid to cid:Cid first before putting it in the blockstore,
            // as the two are different types.
            let multi_cid_bytes: Vec<u8> = multi_cid.into();
            let cid = Cid::try_from(multi_cid_bytes)?;

            blockstore
                .lock()
                .await
                .put_keyed(&cid, &entry_bytes)
                .await?;
        }

        // use fetch_plog_data to get all the data from the blockstore
        let resolver = Resolve { blockstore };

        assert_eq!(plog.entries.len(), 3);

        let (fetched_entries, foot) = fetch_plog_data(plog.head.clone(), resolver.clone()).await?;

        // Reconstruct the plog from the fetched entries
        let first_lock_cid = plog.vlad.cid();

        let entry_bytes = resolver.resolve(first_lock_cid).await?;

        let mayabe_first_lock_script = Script::try_from(entry_bytes.as_slice())?;

        let rebuilt_plog = provenance_log::log::Builder::new()
            // we'll get this from the DHT record key
            .with_vlad(&plog.vlad)
            // First lock script CID is the second half of the vlad
            .with_first_lock(&mayabe_first_lock_script)
            // we get these entries from the network
            .with_entries(&fetched_entries)
            // We will have the head from the DHT record value
            .with_head(&plog.head)
            // foot is the last entry we fecthed from the blockstore
            .with_foot(&foot)
            .try_build()?;

        let verify_iter = &mut rebuilt_plog.verify();

        tracing::info!("*** Verifying rebuilt log **");

        // Entry 0 of the rebuilt plog should match the first entry of the original plog
        assert_eq!(
            plog.entries[&plog.head],
            fetched_entries.get(&plog.head).unwrap().clone()
        );

        // show entry 0 ops (foot entry)
        let mut entries: Vec<&Entry> = rebuilt_plog.entries.values().collect();
        entries.sort();

        let entry_0 = entries[0];

        for op in entry_0.ops() {
            tracing::info!("Entry 0 Op: {:#?}", op);
        }

        // entry get_value for "/entry/" entry_0
        match entry_0.get_value(&Key::try_from("/entry/").unwrap()) {
            Some(value) => {
                tracing::info!("Entry 0 Value: {:#?}", value);
            }
            _ => {
                // fail test
                panic!("Entry 0 has no value for key /entry/");
            }
        }

        // the log should also verify
        for ret in verify_iter {
            match ret {
                Ok((count, entry, kvp)) => {
                    tracing::trace!("Verified entry: {:#?}", entry);
                    tracing::trace!("Verified count: {:#?}", count);
                    tracing::trace!("Verified kvp: {:#?}", kvp);
                }
                Err(e) => {
                    tracing::error!("Error: {:#?}", e);
                    // fail test
                    panic!("Error in log verification");
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
struct Resolve {
    pub blockstore: Arc<Mutex<InMemoryBlockstore<64>>>,
}

impl Resolver for Resolve {
    fn resolve(
        &self,
        cid: &multicid::Cid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>> + Send>> {
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
