pub use mirascript_core::DiagnosticCode;
use mirascript_core::OpCode;
use thiserror::Error;

use super::MiraError;

/// Represents a diagnostic message produced by the compiler.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("{start_line}:{start_col}-{end_line}:{end_col} {code:?}")]
pub struct Diagnostics {
    /// The diagnostic code associated with this message.
    pub code: DiagnosticCode,
    /// The starting line of the diagnostic message (1-based).
    pub start_line: u32,
    /// The starting column of the diagnostic message (1-based).
    pub start_col: u32,
    /// The ending line of the diagnostic message (1-based).
    pub end_line: u32,
    /// The ending column of the diagnostic message (1-based).
    pub end_col: u32,
}

/// Represents the reason for an invalid bytecode error.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InvalidBytecodeReason {
    /// The chunk header is truncated.
    #[error("chunk header is truncated")]
    BadChunkHeader,
    /// The chunk length header does not match the actual chunk length.
    #[error("chunk length header is {0}, expected {1}")]
    ChunkLengthMismatch(usize, usize),
    /// The code section exceeds the chunk length.
    #[error("code section exceeds chunk")]
    CodeSectionExceedsChunk,
    /// The constant section exceeds the chunk length.
    #[error("constant section length exceeds chunk")]
    ConstantSectionExceedsChunk,
    /// The constant section is truncated.
    #[error("truncated constant")]
    TruncatedConstant,
    /// The constant section contains an invalid UTF-8 string.
    #[error("invalid UTF-8 string constant")]
    InvalidStringConstant,
    /// The constant section contains an unknown constant tag.
    #[error("unknown constant tag {0}")]
    UnknownConstantTag(u8),
    /// The root instruction must be a `Func` instruction.
    #[error("root instruction must be Func")]
    BadRootStart,
    /// The last instruction must be a `FuncEnd` instruction.
    #[error("trailing code after root FuncEnd")]
    BadRootEnd,
    /// The root function destination must be register 0.
    #[error("root function destination must be register 0")]
    BadRootDestination,
    /// Unknown opcode encountered in the bytecode.
    #[error("unknown opcode 0x{0:02x}")]
    InvalidOpCode(u8),
    /// The bytecode is truncated while reading an opcode.
    #[error("unexpected end of code while reading opcode")]
    TruncatedOpCode,
    /// The bytecode is truncated while reading an instruction parameter.
    #[error("truncated op parameter at code offset {0}")]
    TruncatedParameter(usize),
    /// The bytecode references a register index that is out of range.
    #[error("register index {0} is out of range 0..={1}")]
    RegisterIndexOutOfRange(usize, usize),
    /// The bytecode references a constant index that is out of range.
    #[error("constant index {0} is out of range 0..{1}")]
    ConstantIndexOutOfRange(usize, usize),
    /// The bytecode references a constant of an invalid type for the instruction.
    #[error("invalid constant type for instruction")]
    InvalidConstantType,
    /// The function instruction has a different number of arguments than the number of registers provided.
    #[error("function has {0} arguments but only {1} registers")]
    FunctionArgCountMismatch(usize, usize),
    /// Unterminated bytecode block encountered while reading an instruction.
    #[error("unterminated bytecode block")]
    UnterminatedBlock,
    /// The bytecode contains an unsupported wide encoding opcode.
    #[error("cannot use wide encoding opcode here")]
    UnsupportedWide,
    /// The bytecode contains an unsupported instruction for the current context.
    #[error("unexpected opcode {0}")]
    UnexpectedOpCode(OpCode),
    /// The module contains a duplicate export key.
    #[error("duplicate export key {0:?}")]
    DuplicateExportKey(String),
    /// The bytecode references an invalid upvalue level.
    #[error("invalid upvalue level {0}")]
    InvalidUpvalueLevel(usize),
    /// The call instruction references an argument index that is out of range for the number of arguments provided.
    #[error("spread argument index {0} is out of range 0..{1}")]
    SpreadArgumentIndexOutOfRange(usize, usize),
}

fn decode_diagnostics(diagnostics: &[u32]) -> Vec<Diagnostics> {
    diagnostics
        .chunks(5)
        .map(|chunk| {
            let start_line = chunk[0] as usize;
            let start_col = chunk[1] as usize;
            let end_line = chunk[2] as usize;
            let end_col = chunk[3] as usize;
            let code =
                DiagnosticCode::from_code(chunk[4] as u16).unwrap_or(DiagnosticCode::Unknown);
            Diagnostics {
                code,
                start_line: start_line as u32,
                start_col: start_col as u32,
                end_line: end_line as u32,
                end_col: end_col as u32,
            }
        })
        .collect()
}

impl MiraError {
    pub(crate) fn invalid_bytecode(offset: usize, reason: InvalidBytecodeReason) -> Box<Self> {
        Self::InvalidBytecode { offset, reason }.into()
    }

    pub(crate) fn compile(diagnostics: &[u32]) -> Box<Self> {
        Self::Compile {
            diagnostics: decode_diagnostics(diagnostics),
        }
        .into()
    }
}
