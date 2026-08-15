use std::fmt;

/// Result type returned by the MiraScript VM.
pub type Result<T> = std::result::Result<T, MiraError>;

/// An error produced while compiling, decoding, executing, or bridging values.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum MiraError {
    /// Source compilation failed.
    Compile {
        /// Compiler diagnostics encoded by `mirascript-core`.
        diagnostics: Vec<u32>,
    },
    /// The compiled bytecode chunk failed structural validation.
    InvalidBytecode {
        /// Byte offset at which validation failed.
        offset: usize,
        /// Human-readable validation failure.
        reason: String,
    },
    /// Script execution failed.
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
    Conversion {
        /// Requested Rust-side value description.
        expected: String,
        /// MiraScript value type that was encountered.
        actual: String,
        /// Nested field or index path, when conversion added one.
        path: Option<String>,
    },
    /// A host extern reported an error.
    Extern {
        /// Error message supplied by the extern.
        message: String,
    },
    /// A live Rust value was already borrowed incompatibly.
    BorrowConflict {
        /// Operation that required the conflicting borrow.
        operation: &'static str,
        /// Type tag of the bridged Rust value.
        tag: String,
    },
    /// Execution exceeded [`crate::RunOptions::timeout`].
    Timeout,
    /// Execution exceeded the configured call-depth limit.
    MaxCallDepth {
        /// Configured maximum call depth.
        max: u32,
    },
    /// A script closure or script module attempted to outlive its execution.
    EscapingClosure,
    /// A previously captured script value was used after execution ended.
    ExecutionEnded,
}

impl MiraError {
    pub(crate) fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
            function: None,
            offset: None,
            stack: Vec::new(),
        }
    }

    pub(crate) fn conversion(expected: impl Into<String>, value: &crate::MiraAny) -> Self {
        Self::Conversion {
            expected: expected.into(),
            actual: value.type_name().into(),
            path: None,
        }
    }

    /// Attach a nested field or index path to a [`MiraError::Conversion`].
    ///
    /// Other error variants are returned unchanged.
    pub fn at_path(mut self, path: impl Into<String>) -> Self {
        if let Self::Conversion { path: slot, .. } = &mut self {
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

impl fmt::Display for MiraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile { diagnostics } => {
                write!(
                    f,
                    "failed to compile MiraScript ({} diagnostic words)",
                    diagnostics.len()
                )
            }
            Self::InvalidBytecode { offset, reason } => {
                write!(f, "invalid bytecode at offset {offset}: {reason}")
            }
            Self::Runtime {
                message,
                function,
                offset,
                ..
            } => {
                write!(f, "{message}")?;
                if let Some(function) = function {
                    write!(f, " in {function}")?;
                }
                if let Some(offset) = offset {
                    write!(f, " at bytecode offset {offset}")?;
                }
                Ok(())
            }
            Self::Conversion {
                expected,
                actual,
                path,
            } => {
                write!(f, "failed to convert {actual} to {expected}")?;
                if let Some(path) = path {
                    write!(f, " at {path}")?;
                }
                Ok(())
            }
            Self::Extern { message } => write!(f, "extern error: {message}"),
            Self::BorrowConflict { operation, tag } => {
                write!(f, "borrow conflict while attempting to {operation} {tag}")
            }
            Self::Timeout => write!(f, "MiraScript execution timed out"),
            Self::MaxCallDepth { max } => write!(f, "maximum call depth of {max} exceeded"),
            Self::EscapingClosure => {
                write!(f, "a MiraScript closure or module cannot escape run()")
            }
            Self::ExecutionEnded => write!(f, "MiraScript execution has ended"),
        }
    }
}

impl std::error::Error for MiraError {}
