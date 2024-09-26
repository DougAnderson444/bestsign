use crate::ops::config::defaults::*;
use crate::ops::config::utils::*;
use crate::ops::config::{LockScript, UnlockScript, VladConfig};

use provenance_log::Script;
use serde::{Deserialize, Serialize};

use crate::ops::update::OpParams;

/// the configuration for opening a new provenance log.
/// It's Serializable and Deserializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The vlad key and cid params
    #[serde(default = "default_vlad_params")]
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
#[derive(Debug, Clone)]
pub struct NewLogBuilder {
    /// Optional Vlad Params
    pub vlad_params: VladConfig,

    /// The entry key params
    pub entrykey_params: OpParams,

    /// The first lock script
    pub first_lock_script: Script,

    /// The entry lock script
    pub entry_lock_script: Script,

    /// The entry unlock script
    pub entry_unlock_script: Script,

    /// Additional ops for the first entry
    pub additional_ops: Vec<OpParams>,

    /// The pubkey params
    pub pubkey_params: OpParams,
}

impl NewLogBuilder {
    /// Create a new ConfigBuilder
    pub fn new(lock: LockScript, unlock: UnlockScript) -> Self {
        Self {
            entry_lock_script: lock.into_inner(),
            entry_unlock_script: unlock.into_inner(),
            vlad_params: default_vlad_params(),
            entrykey_params: default_entrykey_params(),
            first_lock_script: default_first_lock_script(),
            pubkey_params: default_pubkey_params(),
            additional_ops: vec![],
        }
    }

    /// Set the public key params
    pub fn with_pubkey_params(&mut self, params: OpParams) -> &mut Self {
        self.pubkey_params = params;
        self
    }

    /// Set the entry key Params
    pub fn with_entrykey_params(&mut self, params: OpParams) -> &mut Self {
        self.entrykey_params = params;
        self
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
    pub fn with_use_str(
        &mut self,
        params: UseStr,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.additional_ops.push(params.try_into()?);
        Ok(self)
    }

    /// Add a CidGen param Op as an additional op
    pub fn with_cid_gen(&mut self, params: CidGen) -> &mut Self {
        self.additional_ops.push(params.into());
        self
    }

    /// Set the Vlad params
    pub fn with_vlad_params(&mut self, vlad_params: VladConfig) -> &mut Self {
        self.vlad_params = vlad_params;
        self
    }

    /// Build the Config
    pub fn try_build(self) -> Result<Config, Box<dyn std::error::Error>> {
        Ok(Config {
            vlad_params: self.vlad_params,
            entrykey_params: self.entrykey_params,
            pubkey_params: self.pubkey_params,
            first_lock_script: self.first_lock_script,
            entry_lock_script: self.entry_lock_script,
            entry_unlock_script: self.entry_unlock_script,
            additional_ops: self.additional_ops,
        })
    }
}
