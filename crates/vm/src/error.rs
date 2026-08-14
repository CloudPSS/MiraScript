use std::fmt;

pub type Result<T> = std::result::Result<T, MiraError>;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum MiraError {
    Compile {
        diagnostics: Vec<u32>,
    },
    InvalidBytecode {
        offset: usize,
        reason: String,
    },
    Runtime {
        message: String,
        function: Option<String>,
        offset: Option<usize>,
        stack: Vec<String>,
    },
    Conversion {
        expected: String,
        actual: String,
        path: Option<String>,
    },
    Extern {
        message: String,
    },
    BorrowConflict {
        operation: &'static str,
        tag: String,
    },
    Timeout,
    MaxCallDepth {
        max: u32,
    },
    EscapingClosure,
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
