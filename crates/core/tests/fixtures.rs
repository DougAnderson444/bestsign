use bestsign_core::ops::config::defaults::{DEFAULT_PUBKEY, DEFAULT_VLAD_KEY};
use bestsign_core::ops::config::{LockScript, UnlockScript};
use bestsign_core::ops::open::config::NewLogBuilder;
use bestsign_core::ops::{create, CryptoManager};
use bestsign_core::Error;
use multibase::Base;
use multicodec::Codec;
use multihash::EncodedMultihash;
use multikey::{mk, Multikey, Views};
use multisig::Multisig;
use provenance_log::{Key, Log, Script};
use tracing_subscriber::{fmt, EnvFilter};

use serde::{Deserialize, Serialize};

use blockstore::block::{Block, CidError};
use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};

const RAW_CODEC: u64 = 0x55;

pub(crate) fn init_logger() {
    let subscriber = fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        tracing::warn!("failed to set subscriber: {}", e);
    }
}

/// Plog is a wrapper for [Log], so that we can
/// impl [Block] for it.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RawBlock(pub Vec<u8>);

impl Block<64> for RawBlock {
    fn cid(&self) -> Result<Cid, CidError> {
        let mh = Code::Sha2_256.digest(&self.0);
        Ok(Cid::new_v1(RAW_CODEC, mh))
    }

    fn data(&self) -> &[u8] {
        &self.0
    }
}

// helper codec and encode fn
fn encode(mk: &Multikey) -> Result<String, Error> {
    let fp = mk.fingerprint_view()?.fingerprint(Codec::Sha3256)?;
    let ef = EncodedMultihash::new(Base::Base36Lower, fp);
    Ok(ef.to_string())
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TestKeyManager {
    vlad: Option<Multikey>,
    entry_key: Option<Multikey>,
}

impl TestKeyManager {
    pub fn new() -> Self {
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

/// Returns
/// let lock_str = r#"
///        check_signature("/recoverykey", "/entry/") ||
///        check_signature("/pubkey", "/entry/") ||
///        check_preimage("/hash")
/// "#;
pub fn lock_script() -> Script {
    let lock_str = r#"
            check_signature("/recoverykey", "/entry/") ||
            check_signature("/pubkey", "/entry/") ||
            check_preimage("/hash")
        "#;

    Script::Code(Key::default(), lock_str.to_string())
}

/// Returns unlock Script
pub fn unlock_script() -> Script {
    let unlock_str = r#"
                push("/entry/");
                push("/entry/proof");
            "#;

    Script::Code(Key::default(), unlock_str.to_string())
}

// Makes a new plog
pub fn generate_plog() -> Result<Log, Box<dyn std::error::Error>> {
    let lock_script = lock_script();
    let unlock_script = unlock_script();

    let config =
        NewLogBuilder::new(LockScript(lock_script), UnlockScript(unlock_script)).try_build()?;

    let mut key_manager = TestKeyManager::new();

    Ok(create(&config, &mut key_manager)?)
}
