use crate::{InvalidBytecodeReason, MiraAny, MiraError, Result};

pub(super) fn decode_constants(bytes: &[u8], base_offset: usize) -> Result<Vec<MiraAny>> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let tag_offset = offset;
        let tag = bytes[offset];
        offset += 1;
        let value = match tag {
            0 => MiraAny::Nil,
            1 => MiraAny::Boolean(true),
            2 => MiraAny::Boolean(false),
            3 => {
                let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
                    MiraError::invalid_bytecode(
                        base_offset + offset,
                        InvalidBytecodeReason::TruncatedConstant,
                    )
                })?;
                offset += 4;
                MiraAny::Number(i32::from_le_bytes(raw.try_into().expect("checked length")) as f64)
            }
            4 => {
                let raw = bytes.get(offset..offset + 8).ok_or_else(|| {
                    MiraError::invalid_bytecode(
                        base_offset + offset,
                        InvalidBytecodeReason::TruncatedConstant,
                    )
                })?;
                offset += 8;
                MiraAny::Number(f64::from_le_bytes(raw.try_into().expect("checked length")))
            }
            5 => {
                let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
                    MiraError::invalid_bytecode(
                        base_offset + offset,
                        InvalidBytecodeReason::TruncatedConstant,
                    )
                })?;
                let length = u32::from_le_bytes(raw.try_into().expect("checked length")) as usize;
                offset += 4;
                let raw = bytes.get(offset..offset + length).ok_or_else(|| {
                    MiraError::invalid_bytecode(
                        base_offset + offset,
                        InvalidBytecodeReason::TruncatedConstant,
                    )
                })?;
                let value = std::str::from_utf8(raw).map_err(|error| {
                    MiraError::invalid_bytecode(
                        base_offset + offset + error.valid_up_to(),
                        InvalidBytecodeReason::InvalidStringConstant,
                    )
                })?;
                offset += length;
                MiraAny::String(value.into())
            }
            _ => {
                return Err(MiraError::invalid_bytecode(
                    base_offset + tag_offset,
                    InvalidBytecodeReason::UnknownConstantTag(tag),
                ));
            }
        };
        result.push(value);
    }
    Ok(result)
}
