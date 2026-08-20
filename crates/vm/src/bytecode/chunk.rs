use crate::{InvalidBytecodeReason, MiraError, Result};

use super::constants::{Constant, decode_constants};

pub(super) fn decode_chunk(chunk: &[u8]) -> Result<(&[u8], Vec<Constant<'_>>)> {
    if chunk.len() < 12 {
        return Err(MiraError::invalid_bytecode(
            chunk.len(),
            InvalidBytecodeReason::BadChunkHeader,
        ));
    }
    let chunk_len = read_u32(chunk, 0)? as usize;
    if chunk_len != chunk.len() - 4 {
        return Err(MiraError::invalid_bytecode(
            0,
            InvalidBytecodeReason::ChunkLengthMismatch(chunk_len, chunk.len() - 4),
        ));
    }
    let code_len = read_u32(chunk, 4)? as usize;
    let constants_len_offset = 8usize + code_len;
    if constants_len_offset + 4 > chunk.len() {
        return Err(MiraError::invalid_bytecode(
            4,
            InvalidBytecodeReason::ConstantSectionExceedsChunk,
        ));
    }
    let constants_len = read_u32(chunk, constants_len_offset)? as usize;
    let constants_offset = constants_len_offset + 4;
    if constants_offset + constants_len > chunk.len() {
        return Err(MiraError::invalid_bytecode(
            constants_len_offset,
            InvalidBytecodeReason::ConstantSectionExceedsChunk,
        ));
    }
    let constants = decode_constants(&chunk[constants_offset..], constants_offset)?;
    Ok((&chunk[8..8 + code_len], constants))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let data: [u8; 4] = bytes[offset..offset + 4]
        .try_into()
        .expect("checked length");
    Ok(u32::from_le_bytes(data))
}
