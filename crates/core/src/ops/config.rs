mod utils;
use utils::*;
pub use utils::{LockScript, UnlockScript, VladCid, VladConfig};

mod defaults;
use defaults::*;

use std::ops::Deref;

use multicodec::Codec;
use provenance_log::{Key, Script};
use serde::{Deserialize, Serialize};

use crate::ops::update::OpParams;

const DEFAULT_FIRST_LOCK_SCRIPT: &str = r#"check_signature("/entrykey", "/entry/")"#;
const DEFAULT_PUBKEY: &str = "/pubkey";
const DEFAULT_ENTRYKEY: &str = "/entrykey"; // could be ephemeral?
const DEFAULT_VLAD_KEY: &str = "/vlad/key";
const DEFAULT_VLAD_CID: &str = "/vlad/";

/// the configuration for opening a new provenance log.
/// It's Serializable and Deserializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The vlad key and cid params
    pub vlad_params: VladConfig,

    /// The entry key params
    #[serde(default = "default_entrykey_params")]
    pub entrykey_params: OpParams,

    /// The pubkey params
    #[serde(default = "default_pubkey_params")]
    pub pubkey_params: OpParams,

    /// The first lock script
    #[serde(default = "default_first_lock_script")]
    pub first_lock_script: Script,

    /// The entry lock script
    pub entry_lock_script: Script,

    /// The entry unlock script
    pub entry_unlock_script: Script,

    /// Additional ops for the first entry
    pub additional_ops: Vec<OpParams>,
}

/// A Builder for the Config, takes minimal params and allows user to set the rest optionally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBuilder {
    /// Optional Vlad Params
    pub vlad_params: Option<VladConfig>,

    /// The first lock script
    pub first_lock_script: Option<Script>,

    /// The entry lock script
    pub entry_lock_script: Script,

    /// The entry unlock script
    pub entry_unlock_script: Script,

    /// Additional ops for the first entry
    pub additional_ops: Vec<OpParams>,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder
    pub fn new(lock: LockScript, unlock: UnlockScript) -> Self {
        Self {
            vlad_params: Some(default_vlad_params(None, None)),
            first_lock_script: Some(default_first_lock_script()),
            entry_lock_script: lock.into_inner(),
            entry_unlock_script: unlock.into_inner(),
            additional_ops: vec![],
        }
    }

    /// Set the entry lock Script
    pub fn with_entry_lock_script(&mut self, script: Script) -> &mut Self {
        self.entry_lock_script = script;
        self
    }

    /// Set the entry unlock script
    pub fn with_entry_unlock_script(&mut self, script: Script) -> &mut Self {
        self.entry_unlock_script = script;
        self
    }

    /// Add a Key param Op as an additional op
    pub fn with_key_params(&mut self, params: KeyParams) -> &mut Self {
        self.additional_ops.push(params.into());
        self
    }

    /// Add a UseStr param Op as an additional op
    pub fn with_use_str(&mut self, params: UseStr) -> &mut Self {
        self.additional_ops.push(params.into());
        self
    }

    /// Add a CidGen param Op as an additional op
    pub fn with_cid_gen(&mut self, params: CidGen) -> &mut Self {
        self.additional_ops.push(params.into());
        self
    }

    /// Set the Vlad params
    pub fn with_vlad_params(&mut self, vlad_params: VladConfig) -> &mut Self {
        self.vlad_params = Some(vlad_params);
        self
    }

    /// Build the Config
    pub fn try_build(self) -> Result<Config, Box<dyn std::error::Error>> {
        Ok(Config {
            vlad_params: self.vlad_params.unwrap(),
            entrykey_params: default_entrykey_params(),
            pubkey_params: default_pubkey_params(),
            first_lock_script: self.first_lock_script.unwrap(),
            entry_lock_script: self.entry_lock_script,
            entry_unlock_script: self.entry_unlock_script,
            additional_ops: self.additional_ops,
        })
    }
}
