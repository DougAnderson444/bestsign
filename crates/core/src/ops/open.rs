/// Config for the open operation
pub mod config;
use config::Config;
use std::cell::RefCell;

use provenance_log::{multicid, multicodec, multihash, multikey};

pub use crate::ops::config::VladConfig;
use multicid::{cid, vlad};
pub use multicodec::Codec;
use multihash::mh;

use crate::{error::OpenError, utils, Error};
use multikey::{Multikey, Views as _};
use provenance_log::{entry, error::EntryError, Error as PlogError, Log};

use crate::ops::update::OpParams;

use super::traits::CryptoManager;

pub fn create(config: &Config, key_manager: &mut impl CryptoManager) -> Result<Log, crate::Error> {
    // 0. Set up the list of ops we're going to add
    let op_params = RefCell::new(Vec::default());

    let key_manager_ref = RefCell::new(key_manager);

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
            let mk = key_manager_ref
                .borrow_mut()
                .get_mk(key, *codec, *threshold, *limit)?;

            // get the public key
            let pk = if mk.attr_view()?.is_secret_key() {
                mk.conv_view()?.to_public_key()?
            } else {
                mk.clone()
            };

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

    let vlad_mk = load_key(vlad_key_params)?;
    let vlad_cid = load_cid(vlad_cid_params)?;

    // construct the signed vlad using the vlad pubkey and the first lock script cid
    let vlad = vlad::Builder::default()
        .with_cid(&vlad_cid)
        .try_build(|cid| {
            let cv: Vec<u8> = cid.clone().into();
            let ms = key_manager_ref.borrow().prove(&vlad_mk, &cv)?;
            Ok(ms.into())
        })?;

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
            // add the op to the builder
            builder = utils::apply_operations(params, &mut builder)?;
            Ok(())
        })?;

    // finalize the entry building by signing it
    let entry = builder.try_build(|e| {
        // get the serialzied version of the entry with an empty "proof" field
        let ev: Vec<u8> = e.clone().into();
        // call the call back to have the caller sign the data
        let ms = key_manager_ref
            .borrow()
            .prove(&entry_mk, &ev)
            .map_err(|e| PlogError::from(EntryError::SignFailed(e.to_string())))?;
        // store the signature as proof
        Ok(ms.into())
    })?;

    // securely destroy the entry_mk
    drop(entry_mk);

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
    use crate::ops::open::config::NewLogBuilder;

    use super::*;
    use crate::ops::config::defaults::DEFAULT_PUBKEY;
    use crate::ops::config::defaults::DEFAULT_VLAD_KEY;
    use crate::ops::config::{LockScript, UnlockScript};
    use provenance_log::{multibase, multisig};

    use multibase::Base;
    use multicodec::Codec;
    use multihash::EncodedMultihash;
    use multikey::{mk, Multikey};
    use multisig::Multisig;
    use provenance_log::Script;
    use provenance_log::{Key, Pairs};
    use tracing_subscriber::{fmt, EnvFilter};
    use utils::try_extract;

    // helper codec and encode fn
    fn encode(mk: &Multikey) -> Result<String, Error> {
        let fp = mk.fingerprint_view()?.fingerprint(Codec::Sha3256)?;
        let ef = EncodedMultihash::new(Base::Base36Lower, fp);
        Ok(ef.to_string())
    }

    #[derive(Debug, Clone, Default)]
    struct TestKeyManager {
        vlad: Option<Multikey>,
        entry_key: Option<Multikey>,
    }

    impl TestKeyManager {
        fn new() -> Self {
            Self::default()
        }
        /// Returns the vlad key
        pub fn vlad(&self) -> Option<Multikey> {
            self.vlad.clone()
        }
        /// Returns the entry key
        pub fn entry_key(&self) -> Option<Multikey> {
            self.entry_key.clone()
        }
    }

    impl CryptoManager for TestKeyManager {
        fn get_mk(
            &mut self,
            key: &Key,
            codec: Codec,
            _threshold: usize,
            _limit: usize,
        ) -> Result<Multikey, Error> {
            tracing::trace!("Key request for {:?}", key.to_string());

            let mut rng = rand::rngs::OsRng;
            let mk = mk::Builder::new_from_random_bytes(codec, &mut rng)?.try_build()?;

            match key.to_string().as_str() {
                DEFAULT_VLAD_KEY => {
                    // save the public mulitkey for the vlad
                    tracing::trace!(
                        "[GENERATE] {}",
                        encode(&mk).expect("Failed to encode generated MK")
                    );

                    self.vlad = Some(mk.conv_view()?.to_public_key()?);
                    tracing::trace!("Vlad key: {:#?}", self.vlad());
                }
                DEFAULT_PUBKEY => {
                    self.entry_key = Some(mk.clone());
                }
                _ => {}
            }

            Ok(mk)
        }

        fn prove(&self, mk: &Multikey, data: &[u8]) -> Result<Multisig, Error> {
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
            check_signature("/recoverykey", "/entry/") ||
            check_signature("/pubkey", "/entry/") ||
            check_preimage("/hash")
        "#;

        let lock_script = Script::Code(Key::default(), lock_str.to_string());

        let unlock_str = r#"
                push("/entry/");
                push("/entry/proof");
            "#;

        let unlock_script = Script::Code(Key::default(), unlock_str.to_string());

        let config =
            NewLogBuilder::new(LockScript(lock_script), UnlockScript(unlock_script)).build();

        let mut key_manager = TestKeyManager::new();

        let plog = create(&config, &mut key_manager).unwrap();

        tracing::trace!("VLAD key_manager Set?: {:#?}", key_manager.vlad());

        // log.first_lock should match
        assert_eq!(plog.first_lock, config.first_lock_script);

        tracing::trace!("Entries: {:#?}", plog.entries);

        // the entry.vlad is the pubkey against the vlad.nonce as a signature, with the first lock script as the data signed

        // show vlad byte legth
        let vlad_bytes: Vec<u8> = plog.vlad.clone().into();
        tracing::trace!("Vlad [{} bytes]: {:#?}", vlad_bytes.len(), plog.vlad);

        // 1. Get vlad_key from plog first entry
        let verify_iter = &mut plog.verify();

        // the log should also verify
        for ret in verify_iter {
            if let Some(e) = ret.err() {
                tracing::error!("Error: {:#?}", e);
                // fail test
                panic!("Error in log verification");
            }
        }

        // TODO: This API could be improved
        let (_count, _entry, kvp) = &mut plog.verify().next().unwrap().unwrap();

        let vlad_key_value = kvp.get(DEFAULT_VLAD_KEY).unwrap();

        let vlad_key: Multikey = try_extract(&vlad_key_value).unwrap();

        assert_eq!(&vlad_key, &key_manager.vlad().unwrap()); //failing
        assert!(plog.vlad.verify(&vlad_key).is_ok());

        // /pubkey should match key_manager.entry_key public key
        let entry_key = kvp.get(DEFAULT_PUBKEY).unwrap();

        let entry_key: Multikey = try_extract(&entry_key).unwrap();

        assert_eq!(
            entry_key,
            key_manager
                .entry_key()
                .unwrap()
                .conv_view()?
                .to_public_key()?
        );

        Ok(())
    }
}
