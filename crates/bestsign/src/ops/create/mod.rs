/// Config for the open operation
pub mod config;
use std::cell::RefCell;

pub use config::Config;
use config::{VladCid, VladConfig};
use multicid::{cid, vlad};
use multihash::mh;
use multisig::Multisig;

use crate::{error::OpenError, Error};
use multicodec::Codec;
use multikey::{Multikey, Views as _};
use provenance_log::{Key, Log, Script};

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
    config: Config,
    key_manager: impl KeyManager,
    _signer: impl EntrySigner,
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
    for op in config.additional_ops {
        let _ = match op {
            OpParams::KeyGen { .. } => {
                let _ = load_key(&op)?;
                Ok::<(), crate::Error>(())
            }
            OpParams::CidGen { .. } => {
                let _ = load_cid(&op)?;
                Ok(())
            }
            _ => {
                op_params.borrow_mut().push(op.clone());
                Ok(())
            }
        };

        // 1. Construct the VLAD from provided parameters
        let VladConfig {
            key: vlad_key_params,
            cid: vlad_cid_params,
        } = &config.vlad_params.clone();

        // use load key to load the vlad_key and the op_params
        let vlad_mk = load_key(vlad_key_params)?;

        // get the cid for the first lock script, which is config entry_lock_script
        // destructure vlad_cid_params into the Script::Code (r throw Error) to extract the data
        let first_lock_script = match vlad_cid_params {
            VladCid(OpParams::CidGen { data, .. }) => data,
            _ => return Err(OpenError::InvalidKeyParams.into()),
        };

        let vlad_cid = load_cid(vlad_cid_params)?;

        // construct the signed vlad using the vlad pubkey and the first lock script cid
        let vlad = vlad::Builder::default()
            .with_signing_key(&vlad_mk)
            .with_cid(&vlad_cid)
            .try_build()?;

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
    }

    todo!()
}
