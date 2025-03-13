use std::{collections::BTreeMap, future::Future, mem, pin::Pin};

use multibase::Base;
use multicid::{Cid, EncodedCid, EncodedVlad, Vlad};
use multicodec::Codec;
use multihash::EncodedMultihash;
use multikey::{Multikey, Views as _};
use multitrait::Null;
use multiutil::{BaseEncoded, CodecInfo, DetectedEncoder, EncodingInfo};
use provenance_log::{entry, vm, Entry, Key, Log, LogValue, OpId, Pairs, Script};

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

pub trait Resolver {
    fn resolve(
        &self,
        cid: &Cid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>> + Send>>;
}

/// Recursively rsolve data from a head cid down to the foot cid,
/// returning a Vec of Entry for the Plog
/// Also returns the foot Entry
pub async fn fetch_plog_data(
    head_cid: Cid,
    get_data: impl Resolver,
) -> Result<(BTreeMap<multicid::Cid, Entry>, multicid::Cid), Box<dyn std::error::Error>> {
    let mut entries = BTreeMap::new();
    let mut current_cid = head_cid;
    //while current_cid != Null::null() {
    loop {
        let entry_bytes = get_data.resolve(&current_cid).await?;
        let entry = Entry::try_from(entry_bytes.as_slice())?;
        entries.insert(current_cid.clone(), entry.clone());
        if entry.prev() == Null::null() {
            break;
        }
        current_cid = entry.prev();
    }
    let foot = current_cid;

    Ok((entries, foot))
}

/// Vlad details
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VladDetails {
    pub value: Vlad,
    pub bytes: Vec<u8>,
    pub encoded: String,
    pub verified: bool,
}

/// Utility enum to hold all possible display data types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    let vlad_encoded = EncodedVlad::new(Base::Base36Lower, log.vlad.clone()).to_string();
    let vlad_verified = log.vlad.verify(&vlad_key).is_ok();

    //let fingerprint = vlad_key.fingerprint_view()?.fingerprint(Codec::Blake3)?;
    //let ef = EncodedMultihash::new(Base::Base36Lower, fingerprint).to_string();

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
                        let ef = EncodedMultihash::new(Base::Base36Lower, fingerprint).to_string();
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
                            encoded: EncodedCid::new(Base::Base36Lower, cid).to_string(),
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

/// Utility for converting a str slice to Vlad bytes
pub fn decode_vlad(s: &str) -> Result<Vec<u8>, Error> {
    let encoded_vlad = EncodedVlad::try_from(s)?;

    let vlad = encoded_vlad.to_inner();
    Ok(vlad.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlad_from_str() {
        let vlad_str = "kg0nc6q8quc8yyiv9a6u4py2gcmxk8kfcomc3zxy2entacok7w9p6uh8yuq9ogclp9lc2idi8xfdggj4nr1d0u0clrtsj5p6y8tsbvslxzcppoofbg098wek6yrwrjp1bx4nhz5wpsbxkqb0qyclyog8jgbcz3t5v0uju8tmpt3na0c56oz";
        let vlad_bytes = decode_vlad(vlad_str).unwrap();
        assert_eq!(vlad_bytes.len(), 115);

        // can convert back into Vlad
        let vlad = Vlad::try_from(vlad_bytes.as_slice()).unwrap();

        let encoded_vlad = EncodedVlad::new(Base::Base36Lower, vlad).to_string();

        assert_eq!(encoded_vlad, vlad_str);
    }
}
