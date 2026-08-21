mod compile;
mod external;

use thiserror::Error;

use crate::MiraType;

pub use self::compile::{DiagnosticCode, Diagnostics, InvalidBytecodeReason};

/// Result type returned by the MiraScript VM.
pub type Result<T> = std::result::Result<T, Box<MiraError>>;

/// One segment in a failed Rust value conversion path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConversionPathSegment {
    /// A record field.
    Field(String),
    /// An array index.
    Index(usize),
}

/// The reason a MiraScript value could not be converted to a Rust type.
#[derive(Debug, Error)]
pub enum ConversionReason {
    /// The MiraScript value has the wrong category.
    #[error("expected Rust {expected}, got MiraScript {actual}")]
    Type {
        /// Requested Rust type.
        expected: &'static str,
        /// Actual MiraScript type.
        actual: MiraType,
    },
    /// A number cannot be represented by the requested Rust type.
    #[error("number {value} cannot be represented as Rust {expected}")]
    Number {
        /// Requested Rust type.
        expected: &'static str,
        /// Numeric value that failed validation.
        value: f64,
    },
}

/// A VM or standard-library execution failure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RuntimeErrorKind {
    /// A register was read before initialization.
    #[error("uninitialized register value")]
    UninitializedValue,
    /// A non-nil value was required.
    #[error("expected a non-nil value")]
    ExpectedNonNil,
    /// A global variable is not defined.
    #[error("global variable {name:?} is not defined")]
    UndefinedGlobal {
        /// Missing name.
        name: String,
    },
    /// An argument is required.
    #[error("argument {name:?} is required")]
    MissingArgument {
        /// Argument name.
        name: &'static str,
    },
    /// A value has the wrong type for an operation.
    #[error("expected {expected}, got {actual}")]
    TypeMismatch {
        /// Human-readable expected type set.
        expected: &'static str,
        /// Actual value category.
        actual: MiraType,
    },
    /// A value is not callable.
    #[error("value of type {actual} is not callable")]
    NotCallable {
        /// Actual value category.
        actual: MiraType,
    },
    /// An index or field does not exist.
    #[error("index or field does not exist")]
    MissingIndexOrField,
    /// An array allocation would exceed the configured limit.
    #[error("array length {requested} exceeds the configured maximum of {max}")]
    ArrayLimit {
        /// Requested length.
        requested: usize,
        /// Configured maximum.
        max: usize,
    },
    /// An arena handle belongs to another runtime.
    #[error("value handle belongs to another runtime")]
    ForeignHandle,
    /// An arena handle is invalid for its value category.
    #[error("invalid {category} arena handle")]
    InvalidHandle {
        /// Arena category.
        category: &'static str,
    },
    /// The value behind a typed handle has a different concrete Rust type.
    #[error("value behind {category} handle has a different Rust type")]
    HandleTypeMismatch {
        /// Arena category.
        category: &'static str,
    },
    /// One arena category exhausted its 32-bit slot space.
    #[error("{category} arena exhausted its slot space")]
    ArenaExhausted {
        /// Arena category.
        category: &'static str,
    },
    /// Runtime execution was entered recursively.
    #[error("the runtime is already executing a script")]
    ReentrantRun,
    /// Internal bytecode control flow reached an invalid boundary.
    #[error("invalid {context} control flow")]
    InvalidControlFlow {
        /// Boundary being exited.
        context: &'static str,
    },
    /// An upvalue level is invalid for the current frame.
    #[error("invalid upvalue level {level}")]
    InvalidUpvalueLevel {
        /// Requested lexical level.
        level: usize,
    },
    /// Execution exceeded its time limit.
    #[error("execution exceeded the configured timeout")]
    Timeout,
    /// Execution exceeded its call-depth limit.
    #[error("execution exceeded the configured call-depth limit of {max}")]
    MaxCallDepth {
        /// Configured maximum call depth.
        max: u32,
    },
    /// A script closure or script module attempted to escape a run.
    #[error("a script closure or script module attempted to escape its run")]
    EscapingClosure,
    /// A script-scoped callable was used after its run ended.
    #[error("a script-scoped value was used after its run ended")]
    ExecutionEnded,
    /// Bytecode referenced a constant with an incompatible kind.
    #[error("constant {index} is not a {expected} constant")]
    InvalidConstantKind {
        /// Constant-table index.
        index: usize,
        /// Required constant kind.
        expected: &'static str,
    },
    /// Matrix dimensions are invalid or exceed the configured limit.
    #[error("invalid matrix size")]
    InvalidMatrixSize,
    /// A matrix operation requires a square matrix.
    #[error("matrix must be square")]
    MatrixMustBeSquare,
    /// Matrix operands have incompatible dimensions.
    #[error("incompatible matrix dimensions")]
    IncompatibleMatrixDimensions,
    /// An integer-valued argument failed its domain constraint.
    #[error("argument {name:?} must be {constraint}")]
    InvalidIntegerArgument {
        /// Argument name.
        name: &'static str,
        /// Required integer constraint.
        constraint: &'static str,
    },
    /// The number of update entries is not valid.
    #[error("expected an even number of update entries, got {actual}")]
    InvalidUpdateEntryCount {
        /// Actual number of entries.
        actual: usize,
    },
    /// A datetime string could not be parsed.
    #[error("argument \"datetime\" cannot be parsed as a datetime")]
    InvalidDateTime,
    /// A value cannot be interpreted as a timestamp.
    #[error("argument \"datetime\" of type {actual} is not a valid timestamp")]
    InvalidTimestamp {
        /// Actual value category.
        actual: MiraType,
    },
    /// A timezone offset is outside the supported range.
    #[error("argument \"offset\" must be between -24 and 24")]
    TimeOffsetOutOfRange,
    /// JSON serialization failed.
    #[error("failed to serialize JSON: {source}")]
    JsonSerialization {
        /// Serializer error.
        #[source]
        source: serde_json::Error,
    },
    /// JSON input is invalid.
    #[error("invalid JSON: {source}")]
    InvalidJson {
        /// Parser error.
        #[source]
        source: serde_json::Error,
    },
    /// A script assertion or panic supplied a message.
    #[error("{message}")]
    UserMessage {
        /// Script-provided message.
        message: String,
    },
}

/// Runtime source location and call stack attached to an execution error.
#[derive(Debug, Clone, Default)]
pub struct RuntimeTrace {
    /// Function active at the failure site, when known.
    pub function: Option<String>,
    /// Bytecode offset of the failure site, when known.
    pub offset: Option<usize>,
    /// Active callers ordered from root to leaf.
    pub stack: Vec<String>,
}

/// An error produced while compiling, decoding, executing, or bridging values.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MiraError {
    /// The compiled bytecode chunk failed structural validation.
    #[error("invalid bytecode at offset {offset}: {reason}")]
    InvalidBytecode {
        /// Byte offset at which validation failed.
        offset: usize,
        /// Structural validation failure.
        reason: InvalidBytecodeReason,
    },
    /// Source compilation failed.
    #[error("compilation failed with {diagnostics:?}")]
    Compile {
        /// Compiler diagnostics encoded by `mirascript-core`.
        diagnostics: Vec<Diagnostics>,
    },
    /// Script execution failed.
    #[error("runtime failure: {kind}")]
    Runtime {
        /// Structured runtime failure.
        #[source]
        kind: RuntimeErrorKind,
        /// Execution location and stack.
        trace: RuntimeTrace,
    },
    /// A MiraScript value could not be converted to a Rust type.
    #[error("failed to convert value: {reason}")]
    Conversion {
        /// Structured conversion failure.
        #[source]
        reason: ConversionReason,
        /// Nested field and index path.
        path: Vec<ConversionPathSegment>,
    },
    /// A host callback returned an arbitrary error.
    #[error(transparent)]
    External(#[from] anyhow::Error),
}

impl MiraError {
    #[doc(hidden)]
    pub fn runtime(kind: RuntimeErrorKind) -> Box<Self> {
        Box::new(Self::Runtime {
            kind,
            trace: RuntimeTrace::default(),
        })
    }

    pub(crate) fn conversion_type(expected: &'static str, actual: MiraType) -> Box<Self> {
        Box::new(Self::Conversion {
            reason: ConversionReason::Type { expected, actual },
            path: Vec::new(),
        })
    }

    pub(crate) fn conversion_number(expected: &'static str, value: f64) -> Box<Self> {
        Box::new(Self::Conversion {
            reason: ConversionReason::Number { expected, value },
            path: Vec::new(),
        })
    }

    /// Attach a record field to a conversion path.
    pub fn at_field(mut self: Box<Self>, field: impl Into<String>) -> Box<Self> {
        if let Self::Conversion { path, .. } = self.as_mut() {
            path.insert(0, ConversionPathSegment::Field(field.into()));
        }
        self
    }

    /// Attach an array index to a conversion path.
    pub fn at_index(mut self: Box<Self>, index: usize) -> Box<Self> {
        if let Self::Conversion { path, .. } = self.as_mut() {
            path.insert(0, ConversionPathSegment::Index(index));
        }
        self
    }

    pub(crate) fn with_runtime_context(
        mut self,
        function: Option<String>,
        offset: usize,
        stack: Vec<String>,
    ) -> Self {
        if let Self::Runtime { trace, .. } = &mut self
            && trace.offset.is_none()
        {
            trace.function = function;
            trace.offset = Some(offset);
            trace.stack = stack;
        }
        self
    }
}
