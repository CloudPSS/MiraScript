pub use self::compile::{DiagnosticCode, Diagnostics, InvalidBytecodeReason};
use crate::MiraValue;
use thiserror::Error;

mod compile;
mod external;

/// Result type returned by the MiraScript VM.
pub type Result<T> = std::result::Result<T, Box<MiraError>>;

/// An error produced while compiling, decoding, executing, or bridging values.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MiraError {
    /// The compiled bytecode chunk failed structural validation.
    #[error("Invalid bytecode at offset {offset}: {reason}")]
    InvalidBytecode {
        /// Byte offset at which validation failed.
        offset: usize,
        /// Human-readable validation failure.
        reason: InvalidBytecodeReason,
    },
    /// Source compilation failed.
    #[error("Compilation failed with {diagnostics:?}")]
    Compile {
        /// Compiler diagnostics encoded by `mirascript-core`.
        diagnostics: Vec<Diagnostics>,
    },
    /// Script execution failed.
    #[error("Runtime failure: {message}")]
    Runtime {
        /// Human-readable runtime failure.
        message: String,
        /// Function active at the failure site, when known.
        function: Option<String>,
        /// Bytecode offset of the failure site, when known.
        offset: Option<usize>,
        /// Names of active callers, ordered from root to leaf.
        stack: Vec<String>,
    },
    /// A MiraScript value could not be converted to the requested Rust type.
    #[error("Failed to convert {actual} to {expected}")]
    Conversion {
        /// Requested Rust-side value description.
        expected: String,
        /// MiraScript value type that was encountered.
        actual: String,
        /// Nested field or index path, when conversion added one.
        path: Option<String>,
    },
    /// A live Rust value was already borrowed incompatibly.
    #[error(
        "A live Rust value ({tag}) was already borrowed incompatibly by another {operation} operation"
    )]
    BorrowConflict {
        /// Operation that required the conflicting borrow.
        operation: &'static str,
        /// Type tag of the bridged Rust value.
        tag: String,
    },
    /// Execution exceeded [`crate::RunOptions::timeout`].
    #[error("execution exceeded the configured timeout")]
    Timeout,
    /// Execution exceeded the configured call-depth limit.
    #[error("execution exceeded the configured call-depth limit of {max}")]
    MaxCallDepth {
        /// Configured maximum call depth.
        max: u32,
    },
    /// A script closure or script module attempted to outlive its execution.
    #[error("a script closure or script module attempted to outlive its execution")]
    EscapingClosure,
    /// A previously captured script value was used after execution ended.
    #[error("a previously captured script value was used after execution ended")]
    ExecutionEnded,
    /// Trying to access an index or field that does not exist in an array or record.
    #[error("tried to access an index or field that does not exist in an array or record")]
    MissingIndexOrField,
    /// A host extern reported an error.
    #[error(transparent)]
    External(anyhow::Error),
}

impl MiraError {
    pub(crate) fn runtime(message: impl Into<String>) -> Box<Self> {
        Self::Runtime {
            message: message.into(),
            function: None,
            offset: None,
            stack: Vec::new(),
        }
        .into()
    }

    pub(crate) fn conversion(expected: impl Into<String>, value: &MiraValue) -> Box<Self> {
        Self::Conversion {
            expected: expected.into(),
            actual: value.type_name().into(),
            path: None,
        }
        .into()
    }

    /// Attach a nested field or index path to a [`MiraError::Conversion`].
    ///
    /// Other error variants are returned unchanged.
    pub fn at_path(mut self: Box<Self>, path: impl Into<String>) -> Box<Self> {
        if let Self::Conversion { path: slot, .. } = self.as_mut() {
            *slot = Some(path.into());
        }
        self
    }

    pub(crate) fn with_runtime_context(
        mut self,
        function: Option<String>,
        offset: usize,
        stack: Vec<String>,
    ) -> Self {
        if let Self::Runtime {
            function: error_function,
            offset: error_offset,
            stack: error_stack,
            ..
        } = &mut self
            && error_function.is_none()
        {
            *error_function = function;
            *error_offset = Some(offset);
            *error_stack = stack;
        }
        self
    }
}
