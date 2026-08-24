use crate::{InvalidBytecodeReason, MiraError, Result};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Constant {
    Nil,
    True,
    False,
    Int(i32),
    Float(f64),
    String(Box<str>),
}

impl Constant {
    pub(crate) fn to_source_string(&self) -> String {
        match self {
            Self::Nil => String::new(),
            Self::True => true.to_string(),
            Self::False => false.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => crate::operations::number_to_string(*value, false),
            Self::String(value) => value.to_string(),
        }
    }
}

pub(super) fn decode_constants(bytes: &[u8], base_offset: usize) -> Result<Vec<Constant>> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let tag_offset = offset;
        let tag = bytes[offset];
        offset += 1;
        let value = match tag {
            0 => Constant::Nil,
            1 => Constant::True,
            2 => Constant::False,
            3 => {
                let raw = bytes.get(offset..offset + 4).ok_or_else(|| {
                    MiraError::invalid_bytecode(
                        base_offset + offset,
                        InvalidBytecodeReason::TruncatedConstant,
                    )
                })?;
                offset += 4;
                Constant::Int(i32::from_le_bytes(raw.try_into().expect("checked length")))
            }
            4 => {
                let raw = bytes.get(offset..offset + 8).ok_or_else(|| {
                    MiraError::invalid_bytecode(
                        base_offset + offset,
                        InvalidBytecodeReason::TruncatedConstant,
                    )
                })?;
                offset += 8;
                Constant::Float(f64::from_le_bytes(raw.try_into().expect("checked length")))
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
                Constant::String(value.into())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn to_source_string() {
        assert_eq!(Constant::Nil.to_source_string(), "");
        assert_eq!(Constant::True.to_source_string(), "true");
        assert_eq!(Constant::False.to_source_string(), "false");
        assert_eq!(Constant::Int(42).to_source_string(), "42");
        assert_eq!(Constant::Float(3.0).to_source_string(), "3");
        assert_eq!(Constant::Float(-0.0).to_source_string(), "0");
        assert_eq!(Constant::Float(1.2).to_source_string(), "1.2");
        assert_eq!(
            Constant::String("Hello, World!".into()).to_source_string(),
            "Hello, World!"
        );
    }
}
