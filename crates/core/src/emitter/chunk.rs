use super::{
    constant::Constant,
    opcode::{OpCode, OpParam, OpParamTrait},
};

const LEN_SIZE: usize = 4;
const CHUNK_LEN_OFFSET: usize = 0;
const CODE_LEN_OFFSET: usize = CHUNK_LEN_OFFSET + LEN_SIZE;
const CODE_OFFSET: usize = CODE_LEN_OFFSET + LEN_SIZE;

/// A chunk of bytecode.
///
/// Binary format:
/// ```ignore
/// chunk_len: u32 // size of chunk in bytes, excluding this header
/// code_len: u32 // size of code in bytes
/// code: [u8; code_len] // code_len bytes
/// constants_len: u32 // size of constants in bytes
/// constants: [Constant] // constants_len bytes
/// ```
pub struct Chunk<'s> {
    pub constants: Vec<Constant<'s>>,
    pub code: Vec<u8>,
}

impl<'s> Chunk<'s> {
    pub fn new() -> Self {
        Self {
            code: vec![0; CODE_OFFSET],
            constants: Vec::new(),
        }
    }

    /// Create a new chunk with the given code and constants.
    /// Returns Vec<u8> to avoid unnecessary cloning when called by bindings.
    pub fn to_bytes(self) -> Vec<u8> {
        let mut result = self.code;

        let code_len = result.len() - CODE_OFFSET;
        result[CODE_LEN_OFFSET..CODE_LEN_OFFSET + LEN_SIZE]
            .copy_from_slice(&(code_len as u32).to_le_bytes());

        let constants_len_offset = result.len();
        result.extend_from_slice(&[0; LEN_SIZE]);
        for c in self.constants {
            c.write_to(&mut result).unwrap();
        }
        let constants_len = result.len() - constants_len_offset - LEN_SIZE;
        result[constants_len_offset..constants_len_offset + LEN_SIZE]
            .copy_from_slice(&(constants_len as u32).to_le_bytes());

        let chunk_len = result.len() - CHUNK_LEN_OFFSET - LEN_SIZE;
        result[CHUNK_LEN_OFFSET..CHUNK_LEN_OFFSET + LEN_SIZE]
            .copy_from_slice(&(chunk_len as u32).to_le_bytes());

        result
    }

    pub fn add_constant(&mut self, constant: Constant<'s>) -> OpParam {
        if let Some(index) = self.constants.iter().position(|c| c == &constant) {
            OpParam::new(index)
        } else {
            self.constants.push(constant);
            OpParam::new(self.constants.len() - 1)
        }
    }

    pub fn add_code(&mut self, code: OpCode) {
        self.code.push(code.code());
    }

    pub fn add_code_wide(&mut self, code: OpCode) {
        self.code.push(code.wide_code());
    }

    /// Add u8 parameter to the code.
    pub fn add_param(&mut self, param: impl OpParamTrait) {
        self.code.push(param.code());
    }

    /// Add u32 parameter to the code.
    pub fn add_param_wide(&mut self, param: impl OpParamTrait) {
        self.code.extend_from_slice(&param.wide_code());
    }
}
