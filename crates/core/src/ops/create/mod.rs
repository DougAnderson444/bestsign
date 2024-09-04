/// Config for the open operation
pub mod config;
use std::cell::RefCell;

pub use config::Config;
use config::VladConfig;
use multicid::{cid, vlad};
use multihash::mh;
use multisig::Multisig;

use crate::{error::OpenError, ops::update::op, Error};
use multicodec::Codec;
use multikey::{Multikey, Views as _};
use provenance_log::{entry, error::EntryError, Error as PlogError, Key, Log, OpId};

use crate::ops::update::OpParams;

/// Users implement this trait to provide the keys for the log
pub trait KeyManager {
    /// Generate a new key witht he given parameters based on the user's preference for
    /// key generation (new random, from seed, etc.).
    fn generate(
        &self,
        key: &Key,
        codec: Codec,
        start: usize,
        end: usize,
    ) -> Result<Multikey, Error>;
}

/// Users implement this trait to sign the [provenance_log::Entry] for the log
pub trait EntrySigner {
    /// Signs the first Entry in the log with the given ephemeral key
    fn sign(&self, mk: &Multikey, data: &[u8]) -> Result<Multisig, Error>;
}

pub fn create(
    config: &Config,
    key_manager: impl KeyManager,
    signer: impl EntrySigner,
) -> Result<Log, crate::Error> {
    // 0. Set up the list of ops we're going to add
    let op_params = RefCell::new(Vec::default());

    let load_key = |key_params: &OpParams| -> Result<Multikey, crate::Error> {
        if let OpParams::KeyGen {
            key,
            codec,
            threshold,
            limit,
            revoke,
        } = key_params
        {
            // call back to generate the key
            let mk = key_manager.generate(key, *codec, *threshold, *limit)?;

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
            Err(OpenError::InvalidKeyParams.into())
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
    for op in &config.additional_ops {
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

    // 1. Construct the VLAD from provided parameters
    let VladConfig {
        key: vlad_key_params,
        cid: vlad_cid_params,
    } = &config.vlad_params.clone();

    // use load key to load the vlad_key and the op_params
    let vlad_mk = load_key(vlad_key_params)?;

    let vlad_cid = load_cid(vlad_cid_params)?;

    // construct the signed vlad using the vlad pubkey and the first lock script cid
    let vlad = vlad::Builder::default()
        .with_signing_key(&vlad_mk)
        .with_cid(&vlad_cid)
        .try_build()?;

    // drop the vlad_mk to Zeroize the key
    drop(vlad_mk);

    // 2. Call back to get the entry and pub keys and load the lock and unlock scripts

    let entrykey_params = config.entrykey_params.clone();

    // load the entry mk
    let entry_mk = load_key(&entrykey_params)?;

    // get the params for the pubkey
    let pubkey_params = config.pubkey_params.clone();

    // process the pubkey
    let _ = load_key(&pubkey_params)?;

    // the entry lock script
    let lock_script = config.entry_lock_script.clone();

    let unlock_script = config.entry_unlock_script.clone();

    // 3. Construct the first entry, calling back to get the entry signed
    // construct the first entry from all of the parts
    let mut builder = entry::Builder::default()
        .with_vlad(&vlad)
        .add_lock(&lock_script)
        .with_unlock(&unlock_script);
    // add in all of the entry Ops
    op_params
        .borrow_mut()
        .iter()
        .try_for_each(|params| -> Result<(), Error> {
            // construct the op
            let op = match params {
                OpParams::Noop { key } => op::Builder::new(OpId::Noop)
                    .with_key_path(key)
                    .try_build()?,
                OpParams::Delete { key } => op::Builder::new(OpId::Delete)
                    .with_key_path(key)
                    .try_build()?,
                OpParams::UseCid { key, cid } => {
                    let v: Vec<u8> = cid.clone().into();
                    op::Builder::new(OpId::Update)
                        .with_key_path(key)
                        .with_data_value(v)
                        .try_build()?
                }
                OpParams::UseKey { key, mk } => {
                    let v: Vec<u8> = mk.clone().into();
                    op::Builder::new(OpId::Update)
                        .with_key_path(key)
                        .with_data_value(v)
                        .try_build()?
                }
                OpParams::UseStr { key, s } => op::Builder::new(OpId::Update)
                    .with_key_path(key)
                    .with_string_value(s)
                    .try_build()?,
                OpParams::UseBin { key, data } => op::Builder::new(OpId::Update)
                    .with_key_path(key)
                    .with_data_value(data)
                    .try_build()?,
                _ => return Err(OpenError::InvalidOpParams.into()),
            };
            // add the op to the builder
            builder = builder.clone().add_op(&op);
            Ok(())
        })?;

    // finalize the entry building by signing it
    let entry = builder.try_build(|e| {
        // get the serialzied version of the entry with an empty "proof" field
        let ev: Vec<u8> = e.clone().into();
        // call the call back to have the caller sign the data
        let ms = signer
            .sign(&entry_mk, &ev)
            .map_err(|e| PlogError::from(EntryError::SignFailed(e.to_string())))?;
        // store the signature as proof
        Ok(ms.into())
    })?;

    // 4. Construct the log

    let log = provenance_log::log::Builder::new()
        .with_vlad(&vlad)
        .with_first_lock(&config.first_lock_script)
        .append_entry(&entry)
        .try_build()?;

    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::update::OpParams;
    use config::ConfigBuilder;
    use mh::EncodedMultihash;
    use multicodec::Codec;
    use multikey::{mk, Multikey};
    use provenance_log::entry::Entry;
    use provenance_log::log::Log;
    use provenance_log::Script;
    use std::collections::HashMap;
    use tracing_subscriber::{fmt, EnvFilter};
    use vlad::EncodedVlad;

    #[derive(Debug, Clone, Default)]
    struct TestKeyManager;

    impl KeyManager for TestKeyManager {
        fn generate(
            &self,
            _key: &Key,
            codec: Codec,
            _threshold: usize,
            _limit: usize,
        ) -> Result<Multikey, Error> {
            let mut rng = rand::rngs::OsRng;
            let mk = mk::Builder::new_from_random_bytes(codec, &mut rng)?.try_build()?;
            Ok(mk)
        }
    }

    #[derive(Debug, Clone, Default)]
    struct TestSigner;

    impl EntrySigner for TestSigner {
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
        let lock_str = r#"
                check_signature("/recovery", "/entry/") ||
                check_signature("/pubkey", "/entry/") ||
                check_preimage("/hash")
            "#;

        let lock_script = Script::Code(Key::default(), lock_str.to_string());

        let unlock_str = r#"
                push("/entry/");
                push("/entry/proof");
            "#;

        let unlock_script = Script::Code(Key::default(), unlock_str.to_string());

        let config = ConfigBuilder::new(
            config::LockScript(lock_script),
            config::UnlockScript(unlock_script),
        )
        .try_build()?;
        let key_manager = TestKeyManager;
        let signer = TestSigner;

        let plog = create(&config, key_manager, signer).unwrap();

        // TODO: I don't like this API, shows too many internals.
        let plog_clone = plog.clone();

        let verify_iter = plog_clone.verify();

        let mut verified = false;

        for rtn in verify_iter {
            if let Some(e) = rtn.err() {
                tracing::error!("Error: {:#?}", e);
                panic!("Error: {:#?}", e);
            }
        }

        // pretty print the log
        tracing::info!("Vlad: {:#?}", plog.vlad);

        let encoded_vlad = EncodedVlad::from(plog.vlad);

        let s = encoded_vlad.to_string();

        tracing::info!("Encoded Vlad: {}", s);

        // log.first_lock shoul match
        assert_eq!(plog.first_lock, config.first_lock_script);

        Ok(())
    }
}
