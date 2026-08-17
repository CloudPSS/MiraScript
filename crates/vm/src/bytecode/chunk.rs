use crate::{MiraAny, MiraError, Result};

pub(super) fn decode_chunk(chunk: &[u8]) -> Result<(&[u8], Vec<MiraAny>)> {
    if chunk.len() < 12 {
        return Err(MiraError::InvalidBytecode {
            offset: chunk.len(),
            reason: "chunk header is truncated".into(),
        });
    }
    let chunk_len = read_u32(chunk, 0)? as usize;
    if chunk_len != chunk.len() - 4 {
        return Err(MiraError::InvalidBytecode {
            offset: 0,
            reason: format!(
                "chunk length header is {chunk_len}, expected {}",
                chunk.len() - 4
            ),
        });
    }
    let code_len = read_u32(chunk, 4)? as usize;
    let constants_len_offset =
        8usize
            .checked_add(code_len)
            .ok_or_else(|| MiraError::InvalidBytecode {
                offset: 4,
                reason: "code length overflow".into(),
            })?;
    if constants_len_offset
        .checked_add(4)
        .is_none_or(|end| end > chunk.len())
    {
        return Err(MiraError::InvalidBytecode {
            offset: 4,
            reason: "code section exceeds chunk".into(),
        });
    }
    let constants_len = read_u32(chunk, constants_len_offset)? as usize;
    let constants_offset = constants_len_offset + 4;
    if constants_offset
        .checked_add(constants_len)
        .is_none_or(|end| end != chunk.len())
    {
        return Err(MiraError::InvalidBytecode {
            offset: constants_len_offset,
            reason: "constant section length does not match chunk".into(),
        });
    }
    let constants = decode_constants(&chunk[constants_offset..], constants_offset)?;
    Ok((&chunk[8..8 + code_len], constants))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let data: [u8; 4] = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| MiraError::InvalidBytecode {
            offset,
            reason: "truncated u32".into(),
        })?
        .try_into()
        .expect("checked length");
    Ok(u32::from_le_bytes(data))
}

fn decode_constants(bytes: &[u8], base_offset: usize) -> Result<Vec<MiraAny>> {
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
                let raw =
                    bytes
                        .get(offset..offset + 4)
                        .ok_or_else(|| MiraError::InvalidBytecode {
                            offset: base_offset + offset,
                            reason: "truncated ordinal constant".into(),
                        })?;
                offset += 4;
                MiraAny::Number(i32::from_le_bytes(raw.try_into().expect("checked length")) as f64)
            }
            4 => {
                let raw =
                    bytes
                        .get(offset..offset + 8)
                        .ok_or_else(|| MiraError::InvalidBytecode {
                            offset: base_offset + offset,
                            reason: "truncated number constant".into(),
                        })?;
                offset += 8;
                MiraAny::Number(f64::from_le_bytes(raw.try_into().expect("checked length")))
            }
            5 => {
                let raw =
                    bytes
                        .get(offset..offset + 4)
                        .ok_or_else(|| MiraError::InvalidBytecode {
                            offset: base_offset + offset,
                            reason: "truncated string length".into(),
                        })?;
                let length = u32::from_le_bytes(raw.try_into().expect("checked length")) as usize;
                offset += 4;
                let raw = bytes.get(offset..offset + length).ok_or_else(|| {
                    MiraError::InvalidBytecode {
                        offset: base_offset + offset,
                        reason: "truncated string constant".into(),
                    }
                })?;
                let value =
                    std::str::from_utf8(raw).map_err(|error| MiraError::InvalidBytecode {
                        offset: base_offset + offset + error.valid_up_to(),
                        reason: "invalid UTF-8 string constant".into(),
                    })?;
                offset += length;
                MiraAny::String(value.into())
            }
            _ => {
                return Err(MiraError::InvalidBytecode {
                    offset: base_offset + tag_offset,
                    reason: format!("unknown constant tag {tag}"),
                });
            }
        };
        result.push(value);
    }
    Ok(result)
}
