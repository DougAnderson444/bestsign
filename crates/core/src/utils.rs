use std::mem;

use multibase::Base;
use multicid::{Cid, EncodedCid, EncodedVlad, Vlad};
use multicodec::Codec;
use multihash::EncodedMultihash;
use multikey::{Multikey, Views as _};
use multiutil::{BaseEncoded, CodecInfo, DetectedEncoder, EncodingInfo};
use provenance_log::{entry, vm, Key, Log, LogValue, OpId, Pairs, Script};
use serde::{Deserialize, Serialize};

use crate::{
    error::{OpenError, PlogError},
    ops::update::{op, OpParams},
    Error,
};

///  Allies the operations
pub(crate) fn apply_operations(
    params: &OpParams,
    builder: &mut entry::Builder,
) -> Result<entry::Builder, Error> {
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
    let mut latest = builder.clone().add_op(&op);
    Ok(mem::take(&mut latest))
}

/// Extracts an Option<T> the [provenance_log::LogValue]
///
/// where T: TryFrom<&'a [u8]> + EncodingInfo, BaseEncoded<T, DetectedEncoder>: TryFrom<&'a str>
pub fn try_extract<'a, T>(value: &'a vm::Value) -> Option<T>
where
    T: TryFrom<&'a [u8]> + EncodingInfo,
    BaseEncoded<T, DetectedEncoder>: TryFrom<&'a str>,
{
    match value {
        vm::Value::Bin { data, .. } => T::try_from(data.as_slice()).ok(),
        vm::Value::Str { hint: _, data } => {
            BaseEncoded::<T, DetectedEncoder>::try_from(data.as_str())
                .ok()
                .map(|be| be.to_inner())
        }
        _ => None,
    }
}

/// Vlad details
#[derive(Debug, Serialize, Deserialize)]
pub struct VladDetails {
    pub value: Vlad,
    pub bytes: Vec<u8>,
    pub encoded: String,
    pub verified: bool,
}

/// Utility enum to hold all possible display data types
#[derive(Debug, Serialize, Deserialize)]
pub enum DisplayData {
    ReturnValue {
        vlad: VladDetails,
        entries_count: usize,
        kvp_data: Vec<DisplayData>,
    },
    Multikey {
        key_path: Key,
        codec_type: &'static str,
        codec: String,
        fingerprint: String,
    },
    Vlad {
        codec_type: &'static str,
        encoded: String,
        bytes: Vec<u8>,
    },
    Script {
        key_path: Key,
        codec_type: &'static str,
        length: usize,
    },
    Cid {
        key_path: Key,
        codec: String,
        encoded: String,
        codec_type: &'static str,
    },
    Data {
        key_path: Key,
        value: Vec<u8>,
    },
    Str {
        key_path: Key,
        value: String,
    },
    Nil {
        key_path: Key,
    },
}

/// A utility method to extract Key Value pairs from a [provenance_log::Log]
pub fn get_display_data(log: &Log) -> Result<DisplayData, Error> {
    let mut vi = log.verify();
    let (_, _, mut kvp) = vi.next().ok_or::<Error>(PlogError::NoFirstEntry.into())??;

    let vlad_key_value = kvp
        .get("/vlad/key")
        .ok_or::<Error>(PlogError::NoVladKey.into())?;
    let vlad_key: Multikey = try_extract(&vlad_key_value)
        // or InvalidVMValue
        .ok_or::<Error>(PlogError::InvalidVMValue.into())?;

    for ret in vi {
        match ret {
            Ok((_, _, ref pairs)) => kvp = pairs.clone(),
            Err(e) => tracing::debug!("verify failed: {}", e.to_string()),
        }
    }

    let vlad_encoded = EncodedVlad::new(Base::Base32Z, log.vlad.clone()).to_string();
    let vlad_verified = log.vlad.verify(&vlad_key).is_ok();

    let fingerprint = vlad_key.fingerprint_view()?.fingerprint(Codec::Blake3)?;
    let ef = EncodedMultihash::new(Base::Base32Z, fingerprint).to_string();

    let kvp_data = kvp
        .iter()
        .map(|(k, val)| -> Result<DisplayData, Error> {
            let value = if let Some(codec) = get_codec_from_plog_value(val) {
                let v = kvp
                    .get(k.as_str())
                    .ok_or::<Error>(PlogError::NoKeyPath.into())?;
                match codec {
                    Codec::Multikey => {
                        let key: Multikey =
                            try_extract(&v).ok_or::<Error>(PlogError::InvalidVMValue.into())?;
                        let fingerprint = key.fingerprint_view()?.fingerprint(Codec::Blake3)?;
                        let ef = EncodedMultihash::new(Base::Base32Z, fingerprint).to_string();
                        DisplayData::Multikey {
                            key_path: k.clone(),
                            codec_type: codec.into(),
                            codec: key.codec().to_string(),
                            fingerprint: ef,
                        }
                    }
                    Codec::Vlad => {
                        let vlad: Vlad =
                            try_extract(&v).ok_or::<Error>(PlogError::InvalidVMValue.into())?;
                        let bytes: Vec<u8> = vlad.clone().into();
                        DisplayData::Vlad {
                            codec_type: codec.into(),
                            encoded: EncodedVlad::new(Base::Base36Lower, vlad).to_string(),
                            bytes,
                        }
                    }
                    Codec::ProvenanceLogScript => {
                        let script: Script =
                            try_extract(&v).ok_or::<Error>(PlogError::InvalidVMValue.into())?;
                        DisplayData::Script {
                            key_path: k.clone(),
                            codec_type: codec.into(),
                            length: script.as_ref().len(),
                        }
                    }
                    Codec::Cidv1 | Codec::Cidv2 | Codec::Cidv3 => {
                        let cid: Cid =
                            try_extract(&v).ok_or::<Error>(PlogError::InvalidVMValue.into())?;
                        DisplayData::Cid {
                            key_path: k.clone(),
                            codec: cid.codec().to_string(),
                            encoded: EncodedCid::new(Base::Base32Z, cid).to_string(),
                            codec_type: codec.into(),
                        }
                    }
                    _ => DisplayData::Nil {
                        key_path: k.clone(),
                    },
                }
            } else {
                match val {
                    LogValue::Data(v) => DisplayData::Data {
                        key_path: k.clone(),
                        value: v.to_vec(),
                    },
                    LogValue::Str(s) => DisplayData::Str {
                        key_path: k.clone(),
                        value: s.to_string(),
                    },
                    _ => DisplayData::Nil {
                        key_path: k.clone(),
                    },
                }
            };
            Ok(value)
        })
        .collect::<Result<Vec<DisplayData>, Error>>()?;

    let display_data = DisplayData::ReturnValue {
        vlad: VladDetails {
            value: log.vlad.clone(),
            bytes: log.vlad.clone().into(),
            encoded: vlad_encoded,
            verified: vlad_verified,
        },
        entries_count: log.entries.len(),
        kvp_data,
    };

    Ok(display_data)
}

/// Utility method to extract a [Codec] from a [provenance_log::LogValue]
fn get_codec_from_plog_value(value: &LogValue) -> Option<Codec> {
    match value {
        LogValue::Data(v) => Codec::try_from(v.as_slice()).ok(),
        LogValue::Str(s) => Codec::try_from(s.as_str()).ok(),
        _ => None,
    }
}
