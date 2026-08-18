pub use self::compile::{DiagnosticCode, Diagnostics, InvalidBytecodeReason};
use thiserror::Error;

mod compile;

/// Result type returned by the MiraScript VM.
pub type Result<T> = std::result::Result<T, Box<MiraError>>;

/// An error produced while compiling, decoding, executing, or bridging values.
#[derive(Debug, Clone, PartialEq, Error)]
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
    #[error("")]
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
    #[error("")]
    Conversion {
        /// Requested Rust-side value description.
        expected: String,
        /// MiraScript value type that was encountered.
        actual: String,
        /// Nested field or index path, when conversion added one.
        path: Option<String>,
    },
    /// A host extern reported an error.
    #[error("")]
    Extern {
        /// Error message supplied by the extern.
        message: String,
    },
    /// A live Rust value was already borrowed incompatibly.
    #[error("")]
    BorrowConflict {
        /// Operation that required the conflicting borrow.
        operation: &'static str,
        /// Type tag of the bridged Rust value.
        tag: String,
    },
    /// Execution exceeded [`crate::RunOptions::timeout`].
    #[error("")]
    Timeout,
    /// Execution exceeded the configured call-depth limit.
    #[error("")]
    MaxCallDepth {
        /// Configured maximum call depth.
        max: u32,
    },
    /// A script closure or script module attempted to outlive its execution.
    #[error("")]
    EscapingClosure,
    /// A previously captured script value was used after execution ended.
    #[error("")]
    ExecutionEnded,
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

    pub(crate) fn conversion(expected: impl Into<String>, value: &crate::MiraAny) -> Box<Self> {
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
