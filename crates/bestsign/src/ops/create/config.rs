use std::ops::Deref;

use multicodec::Codec;
use provenance_log::{Key, Script};
use serde::{Deserialize, Serialize};

use crate::ops::update::OpParams;

/// NewType Wrapper around VladKey OpParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VladKey(pub OpParams);

/// NewType Wrapper around VladCid OpParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VladCid(pub OpParams);

/// NewType wrapper around KeyCodec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCodec(pub Codec);

/// NewType wrapper around HashCodec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashCodec(pub Codec);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyParams {
    pub codec: KeyCodec,
    pub threshold: Option<usize>,
    pub limit: Option<usize>,
}

impl Deref for VladKey {
    type Target = OpParams;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for VladCid {
    type Target = OpParams;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for KeyCodec {
    type Target = Codec;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for HashCodec {
    type Target = Codec;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A struct for the Key Params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyParams {
    pub key: Key,
    pub codec: KeyCodec,
    pub threshold: Option<usize>,
    pub limit: Option<usize>,
    pub revoke: Option<bool>,
}

/// From KeyParams to OpParams
impl From<KeyParams> for OpParams {
    fn from(params: KeyParams) -> Self {
        OpParams::KeyGen {
            key: params.key,
            codec: *params.codec,
            threshold: params.threshold.unwrap_or_default(),
            limit: params.limit.unwrap_or_default(),
            revoke: params.revoke.unwrap_or_default(),
        }
    }
}

// UseStr
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseStr {
    pub key: Key,
    pub s: String,
}

impl From<UseStr> for OpParams {
    fn from(params: UseStr) -> Self {
        OpParams::UseStr {
            key: params.key,
            s: params.s,
        }
    }
}

// CidGen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CidGen {
    pub key: Key,
    pub version: Codec,
    pub target: Codec,
    pub hash: HashCodec,
    pub inline: bool,
}

impl From<CidGen> for OpParams {
    fn from(params: CidGen) -> Self {
        OpParams::CidGen {
            key: params.key,
            version: params.version,
            target: params.target,
            hash: *params.hash,
            inline: params.inline,
        }
    }
}

/// the configuration for opening a new provenance log.
/// It's Serializable and Deserializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The vlad key and cid params
    pub vlad_params: (VladKey, VladCid),

    /// The entry key params
    #[serde(default = "default_entrykey_params")]
    pub entrykey_params: OpParams,

    /// The pubkey params
    #[serde(default = "default_pubkey_params")]
    pub pubkey_params: OpParams,

    /// The entry lock script
    pub entry_lock_script: Script,

    /// The entry unlock script
    pub entry_unlock_script: Script,

    /// Additional ops for the first entry
    pub additional_ops: Vec<OpParams>,
}

/// A Builder for the Config, takes minimal params and allows user to set the rest optionally
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigBuilder {
    /// Optional Vlad Params
    pub vlad_params: Option<(VladKey, VladCid)>,

    /// The entry lock script
    pub entry_lock_script: Option<Script>,

    /// The entry unlock script
    pub entry_unlock_script: Option<Script>,

    /// Additional ops for the first entry
    pub additional_ops: Vec<OpParams>,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder
    pub fn new() -> Self {
        Self {
            vlad_params: Some(default_vlad_params(None, None)),
            entry_lock_script: None,
            entry_unlock_script: None,
            additional_ops: vec![],
        }
    }

    /// Set the entry lock Script
    pub fn with_entry_lock_script(&mut self, script: Script) -> &mut Self {
        self.entry_lock_script = Some(script);
        self
    }

    /// Set the entry unlock script
    pub fn with_entry_unlock_script(&mut self, script: Script) -> &mut Self {
        self.entry_unlock_script = Some(script);
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
    pub fn with_vlad_params(&mut self, vlad_params: (VladKey, VladCid)) -> &mut Self {
        self.vlad_params = Some(vlad_params);
        self
    }

    /// Build the Config
    pub fn try_build(self) -> Result<Config, Box<dyn std::error::Error>> {
        // if any missing params, return a missing error
        if self.entry_lock_script.is_none() {
            return Err("Missing entry lock script".into());
        }

        if self.entry_unlock_script.is_none() {
            return Err("Missing entry unlock script".into());
        }

        Ok(Config {
            vlad_params: self.vlad_params.unwrap(),
            entrykey_params: default_entrykey_params(),
            pubkey_params: default_pubkey_params(),
            entry_lock_script: self.entry_lock_script.unwrap(),
            entry_unlock_script: self.entry_unlock_script.unwrap(),
            additional_ops: self.additional_ops,
        })
    }
}

fn default_pubkey_params() -> OpParams {
    OpParams::KeyGen {
        key: Key::try_from("/pubkey").unwrap(),
        codec: Codec::Ed25519Priv,
        threshold: 0,
        limit: 0,
        revoke: false,
    }
}

/// The default entry key params is /entrykey"
pub fn default_entrykey_params() -> OpParams {
    OpParams::KeyGen {
        key: Key::try_from("/entrykey").unwrap(),
        codec: Codec::Ed25519Priv,
        threshold: 0,
        limit: 0,
        revoke: false,
    }
}

/// Creates the VladParasm tuples from a given [Script] and Optional [Codec] (uses Ed25519 by default)
pub fn default_vlad_params(
    key_codec: Option<KeyCodec>,
    hash_codec: Option<HashCodec>,
) -> (VladKey, VladCid) {
    let key_codec = KeyCodec(*key_codec.unwrap_or(KeyCodec(Codec::Ed25519Priv)));
    let hash_codec = HashCodec(*hash_codec.unwrap_or(HashCodec(Codec::Blake3)));
    (
        VladKey(OpParams::KeyGen {
            key: Key::try_from("/vlad/key").unwrap(),
            codec: *key_codec,
            threshold: 0,
            limit: 0,
            revoke: false,
        }),
        VladCid(OpParams::CidGen {
            key: Key::try_from("/vlad/").unwrap(),
            version: Codec::Cidv1,
            target: Codec::Identity,
            hash: *hash_codec,
            inline: true,
        }),
    )
}
