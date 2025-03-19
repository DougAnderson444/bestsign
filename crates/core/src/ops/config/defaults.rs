// SPDX-License-Identifier: FSL-1.1
// Copyright 2024 Doug Anderson
//! Default values for the [OpParams] and [Script] structs
use provenance_log::multicodec;

use multicodec::Codec;
use provenance_log::{Key, Script};

use crate::ops::update::OpParams;

use super::*;

/// The default entry key, "/entrykey"
pub const DEFAULT_ENTRYKEY: &str = "/entrykey";

/// The default first lock script
///
/// ```rhai
/// check_signature("/entrykey", "/entry/")
/// ```
pub const DEFAULT_FIRST_LOCK_SCRIPT: &str = r#"check_signature("/entrykey", "/entry/")"#;

/// The default pubkey key, "/pubkey"
pub const DEFAULT_PUBKEY: &str = "/pubkey";

/// The default Vlad key, "/vlad/key"
pub const DEFAULT_VLAD_KEY: &str = "/vlad/key";

/// The default Vlad CID Key, "/vlad/"
pub const DEFAULT_VLAD_CID: &str = "/vlad/";

pub(crate) fn default_pubkey_params() -> OpParams {
    OpParams::KeyGen {
        key: Key::try_from(DEFAULT_PUBKEY).unwrap(),
        codec: Codec::Ed25519Priv,
        threshold: 1,
        limit: 1,
        revoke: false,
    }
}

/// The default entry key params
pub(crate) fn default_entrykey_params() -> OpParams {
    OpParams::KeyGen {
        key: Key::try_from(DEFAULT_ENTRYKEY).unwrap(),
        codec: Codec::Ed25519Priv,
        threshold: 1,
        limit: 1,
        revoke: false,
    }
}

/// The default first entry lock script
pub(crate) fn default_first_lock_script() -> Script {
    Script::Code(Key::default(), DEFAULT_FIRST_LOCK_SCRIPT.to_string())
}

/// Creates the VladParasm tuples from a given [Script] and Optional [Codec] (uses Ed25519 by default)
pub(crate) fn default_vlad_params() -> VladConfig {
    let key_codec = KeyCodec(Codec::Ed25519Priv);
    let hash_codec = HashCodec(Codec::Blake3);
    VladConfig {
        key: VladKey(OpParams::KeyGen {
            key: Key::try_from(DEFAULT_VLAD_KEY).unwrap(),
            codec: *key_codec,
            threshold: 0,
            limit: 0,
            revoke: false,
        }),
        cid: VladCid(OpParams::CidGen {
            key: Key::try_from(DEFAULT_VLAD_CID).unwrap(),
            version: Codec::Cidv1,
            target: Codec::Identity,
            hash: *hash_codec,
            inline: true,
            // Data is the bytes of DEFAULT_FIRST_LOCK_SCRIPT value
            data: DEFAULT_FIRST_LOCK_SCRIPT.as_bytes().to_vec(),
        }),
    }
}
