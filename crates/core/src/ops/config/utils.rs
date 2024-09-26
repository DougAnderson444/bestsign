use std::ops::Deref;

use multicodec::Codec;
use provenance_log::{Key, Script};

use crate::{ops::update::OpParams, Error};

use super::*;

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

/// Add a Key to the OpParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyParams {
    /// How you identify this key, ie. "/pubkey" or "/key1"
    pub key: Key,
    /// The codec to use for the key, ie. "ed25519" or "secp256k1"
    pub codec: KeyCodec,
    /// Optional threshold for threshold key splitting
    pub threshold: Option<usize>,
    /// Optional limit for threshold key splitting
    pub limit: Option<usize>,
    /// Optional flag to revoke the previous key
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

/// Add a Key Value<String>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseStr {
    pub key: String,
    pub value: String,
}

impl TryFrom<UseStr> for OpParams {
    type Error = Error;

    fn try_from(params: UseStr) -> Result<Self, Self::Error> {
        Ok(OpParams::UseStr {
            key: Key::try_from(params.key)?,
            s: params.value,
        })
    }
}

// Add a Content Identifier (CID) to the OpParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CidGen {
    pub key: Key,
    pub version: Codec,
    pub target: Codec,
    pub hash: HashCodec,
    pub inline: bool,
    pub data: Vec<u8>,
}

impl From<CidGen> for OpParams {
    fn from(params: CidGen) -> Self {
        OpParams::CidGen {
            key: params.key,
            version: params.version,
            target: params.target,
            hash: *params.hash,
            inline: params.inline,
            data: params.data,
        }
    }
}

/// Vlad config struct, made up of VladKey and VladCid fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VladConfig {
    pub key: VladKey,
    pub cid: VladCid,
}

/// NewType Wrapper around Lock Script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockScript(pub Script);

impl LockScript {
    /// Moves the value out of the LockScript
    pub fn into_inner(self) -> Script {
        self.0
    }
}

impl Deref for LockScript {
    type Target = Script;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// NewType Wrapper around Unlock script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockScript(pub Script);

impl UnlockScript {
    /// Moves the value out of the UnlockScript
    pub fn into_inner(self) -> Script {
        self.0
    }
}

impl Deref for UnlockScript {
    type Target = Script;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
