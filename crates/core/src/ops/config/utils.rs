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
