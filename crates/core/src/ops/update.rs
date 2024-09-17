pub mod config;
use std::cell::RefCell;

pub use config::UpdateConfig;

/// Utilities for building plog Update Ops
pub mod op;

/// params for generating Op
pub mod op_params;
use multicid::cid;
use multihash::mh;
use multikey::{Multikey, Views as _};
pub use op_params::OpParams;
use provenance_log::error::EntryError;
use provenance_log::Entry;
use provenance_log::{entry, error::Error as PlogError, Log};

use crate::{
    error::OpenError,
    ops::traits::{EntrySigner, KeyManager},
    utils, Error,
};

pub fn update_plog(
    plog: &mut Log,
    config: &UpdateConfig,
    key_manager: &mut (impl KeyManager + EntrySigner),
) -> Result<Entry, crate::Error> {
    // 0. Set up the list of ops we're going to add
    let op_params = RefCell::new(Vec::default());

    let mut load_key = |key_params: &OpParams| -> Result<Multikey, crate::Error> {
        if let OpParams::KeyGen {
            key,
            codec,
            threshold,
            limit,
            revoke,
        } = key_params
        {
            // call back to generate the key
            let mk = key_manager.get_mk(key, *codec, *threshold, *limit)?;

            // get the public key
            let pk = mk.conv_view()?.to_public_key()?;

            // if revoking, explicitly delete the old key first
            if *revoke {
                op_params
                    .borrow_mut()
                    .push(OpParams::Delete { key: key.clone() });
            }

            // add the op params to add the key
            op_params.borrow_mut().push(OpParams::UseKey {
                key: key.clone(),
                mk: pk,
            });

            Ok(mk)
        } else {
            Err(crate::error::OpenError::InvalidKeyParams.into())
        }
    };

    let load_cid = |cid_params: &OpParams| -> Result<cid::Cid, crate::Error> {
        if let OpParams::CidGen {
            key,
            version,
            target,
            hash,
            inline,
            data,
        } = cid_params
        {
            let cid = cid::Builder::new(*version)
                .with_target_codec(*target)
                .with_hash(&mh::Builder::new_from_bytes(*hash, data)?.try_build()?)
                .try_build()?;

            // create the cid key-path
            let mut cid_key = key.clone();
            cid_key.push("/cid")?;

            // add the op params to add the cid for the file
            op_params.borrow_mut().push(OpParams::UseCid {
                key: cid_key,
                cid: cid.clone(),
            });

            // add the file directly to p.log if inline
            if *inline {
                // create the cid key-path
                let mut data_key = key.clone();
                data_key.push("/data")?;

                // add the op param to add the file data
                op_params.borrow_mut().push(OpParams::UseBin {
                    key: data_key,
                    data: data.to_vec(),
                });
            }

            Ok(cid)
        } else {
            Err(OpenError::InvalidKeyParams.into())
        }
    };

    // go through the additional ops and generate CIDs and keys and adding the resulting op params
    // to the vec of op params
    for op in &config.entry_ops {
        let _ = match op {
            OpParams::KeyGen { .. } => {
                let _ = load_key(op)?;
                Ok::<(), crate::Error>(())
            }
            OpParams::CidGen { .. } => {
                let _ = load_cid(op)?;
                Ok(())
            }
            _ => {
                op_params.borrow_mut().push(op.clone());
                Ok(())
            }
        };
    }

    // 1. validate the p.log and get the last entry and state
    let (_, last_entry, _kvp) = plog
        .verify()
        .last()
        .ok_or(crate::error::UpdateError::NoLastEntry)??;

    let unlock_script = config.entry_unlock_script.clone();
    let entry_mk = config.entry_signing_key.clone();

    // construct the first entry from all of the parts
    let mut builder = entry::Builder::from(&last_entry).with_unlock(&unlock_script);

    for (_key_path, lock) in &config.add_entry_lock_scripts {
        builder = builder.add_lock(lock);
    }

    // add in all of the entry Ops
    op_params
        .borrow_mut()
        .iter()
        .try_for_each(|params| -> Result<(), Error> {
            // add the op to the builder
            builder = utils::apply_operations(params, &mut builder)?;
            Ok(())
        })?;
    // finalize the entry building by signing it
    let entry = builder.try_build(|e| {
        // get the serialzied version of the entry with an empty "proof" field
        let ev: Vec<u8> = e.clone().into();
        // call the call back to have the caller sign the data
        let ms = key_manager
            .sign(&entry_mk, &ev)
            .map_err(|e| PlogError::from(EntryError::SignFailed(e.to_string())))?;
        // store the signature as proof
        Ok(ms.into())
    })?;

    // try to add the entry to the p.log
    plog.try_append(&entry)?;

    Ok(entry)
}

#[cfg(test)]
mod tests {
    use crate::ops::{
        config::{
            defaults::{DEFAULT_ENTRYKEY, DEFAULT_PUBKEY},
            LockScript, UnlockScript,
        },
        create,
        open::config::ConfigBuilder,
    };

    use super::*;
    use multicodec::Codec;
    use multikey::{mk, Multikey};
    use multisig::Multisig;
    use provenance_log::{Key, Pairs, Script};
    use tracing_subscriber::{fmt, EnvFilter};

    #[derive(Debug, Clone, Default)]
    struct TestKeyManager {
        /// The "/pubkey" key
        key: Option<Multikey>,
    }

    impl TestKeyManager {
        fn new() -> Self {
            Self { key: None }
        }
    }

    impl KeyManager for TestKeyManager {
        fn get_mk(
            &mut self,
            key: &Key,
            codec: Codec,
            _threshold: usize,
            _limit: usize,
        ) -> Result<Multikey, Error> {
            tracing::info!(
                "Key request for {:?} with codec {:#?}",
                key.to_string(),
                codec
            );
            let mut rng = rand::rngs::OsRng;
            let mk = mk::Builder::new_from_random_bytes(codec, &mut rng)?.try_build()?;

            // if Key is "/pubkey" then set the key
            if key.to_string() == DEFAULT_PUBKEY {
                self.key = Some(mk.clone());
            }
            Ok(mk)
        }
    }

    impl EntrySigner for TestKeyManager {
        fn sign(&self, mk: &Multikey, data: &[u8]) -> Result<Multisig, Error> {
            Ok(mk.sign_view()?.sign(data, false, None)?)
        }
    }

    fn init_logger() {
        let subscriber = fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            tracing::warn!("failed to set subscriber: {}", e);
        }
    }

    #[test]
    fn test_create_using_defaults() -> Result<(), Box<dyn std::error::Error>> {
        init_logger();

        // 1. Create a new p.log with the default Config
        let lock = r#"
            check_signature("/recoverykey", "/entry/") ||
            check_signature("/pubkey", "/entry/") ||
            check_preimage("/hash")
        "#
        .to_string();

        let lock_script = Script::Code(Key::default(), lock);

        let unlock_str = r#"
                push("/entry/");
                push("/entry/proof");
            "#;

        let unlock_script = Script::Code(Key::default(), unlock_str.to_string());

        let config = ConfigBuilder::new(
            LockScript(lock_script.clone()),
            UnlockScript(unlock_script.clone()),
        )
        .try_build()?;

        let mut key_manager = TestKeyManager::new();

        let mut plog = create(&config, &mut key_manager).unwrap();

        // 2. Update the p.log with a new entry
        // - add a lock Script
        // - remove the entrykey lock Script
        // - add an op
        let update_cfg = UpdateConfig::new(
            unlock_script.clone(),
            key_manager.key.clone().unwrap_or_default(),
        )
        .add_op(OpParams::Delete {
            key: Key::try_from(DEFAULT_ENTRYKEY).unwrap(),
        })
        // Entry lock scripts define conditions which must be met by the next entry in the plog for it to be valid.
        .add_lock(Key::try_from("/delegated/")?, lock_script);

        // tak config and use update method with TestKeyManager to update the log
        update_plog(&mut plog, &update_cfg, &mut key_manager)?;

        // There should be no DEFAULT_ENTRYKEY kvp
        let verify_iter = &mut plog.verify();

        let mut last = None;

        // the log should also verify
        for ret in verify_iter {
            if let Some(e) = ret.clone().err() {
                tracing::error!("Error: {:#?}", e);
                // fail test
                panic!("Error in log verification");
            } else {
                last = Some(ret.ok().unwrap());
            }
        }

        let (count, entry, kvp) = last.ok_or("No last entry")?;
        let op = kvp.get(DEFAULT_ENTRYKEY);

        assert!(op.is_none());

        Ok(())
    }
}
