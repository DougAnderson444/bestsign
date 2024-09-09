use std::mem;

use multiutil::{BaseEncoded, DetectedEncoder, EncodingInfo};
use provenance_log::{entry, vm, OpId};

use crate::{
    error::OpenError,
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
pub(crate) fn try_extract<'a, T>(value: &'a vm::Value) -> Option<T>
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
