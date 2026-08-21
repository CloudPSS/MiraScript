use crate::interpreter::RegisterId;

use super::constants::Constant;

#[derive(Debug)]
pub(crate) struct Program {
    pub constants: Box<[Constant]>,
    pub global_names: Box<[(String, Option<usize>)]>,
    pub root: FunctionDef,
    pub functions: Box<[FunctionDef]>,
}

#[derive(Debug)]
pub(crate) struct FunctionDef {
    #[allow(dead_code)] // Retained for function-level diagnostics and future source maps.
    pub offset: usize,
    pub arg_count: usize,
    pub register_count: usize,
    pub variadic: bool,
    pub body: Box<[Instruction]>,
}

#[derive(Debug)]
pub(crate) struct Instruction {
    pub offset: usize,
    pub kind: InstructionKind,
}

#[derive(Debug)]
pub(crate) enum InstructionKind {
    Op(Operation),
    Function {
        destination: RegisterId,
        function: usize,
    },
    If {
        condition: Condition,
        register: RegisterId,
        then_body: Box<[Instruction]>,
        else_body: Box<[Instruction]>,
    },
    Loop {
        register_count: usize,
        kind: LoopKind,
        body: Box<[Instruction]>,
        reuse_frame: bool,
    },
    Record {
        destination: RegisterId,
        elements: Vec<RecordElement>,
    },
    Array {
        destination: RegisterId,
        elements: Vec<ArrayElement>,
    },
    Module {
        destination: RegisterId,
        name: String,
        fields: Vec<(String, RegisterId)>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Noop,
    Break,
    Continue,
    Return {
        value: RegisterId,
    },
    Constant {
        destination: RegisterId,
        constant: usize,
    },
    Uninit {
        destination: RegisterId,
    },
    Unary {
        kind: UnaryOperation,
        destination: RegisterId,
        value: RegisterId,
    },
    Add {
        destination: RegisterId,
        left: RegisterId,
        right: RegisterId,
    },
    Sub {
        destination: RegisterId,
        left: RegisterId,
        right: RegisterId,
    },
    Mul {
        destination: RegisterId,
        left: RegisterId,
        right: RegisterId,
    },
    Div {
        destination: RegisterId,
        left: RegisterId,
        right: RegisterId,
    },
    Mod {
        destination: RegisterId,
        left: RegisterId,
        right: RegisterId,
    },
    Pow {
        destination: RegisterId,
        left: RegisterId,
        right: RegisterId,
    },
    Binary {
        kind: BinaryOperation,
        destination: RegisterId,
        left: RegisterId,
        right: RegisterId,
    },
    Swap {
        left: RegisterId,
        right: RegisterId,
    },
    Upvalue {
        kind: UpvalueOperation,
        value: RegisterId,
        level: usize,
        register: RegisterId,
    },
    GetGlobal {
        destination: RegisterId,
        slot: usize,
    },
    GetGlobalDyn {
        destination: RegisterId,
        key: RegisterId,
    },
    InGlobal {
        destination: RegisterId,
        key: RegisterId,
    },
    Concat {
        destination: RegisterId,
        values: Box<[RegisterId]>,
    },
    Format {
        destination: RegisterId,
        value: RegisterId,
        format: usize,
    },
    Assert {
        kind: AssertOperation,
        value: RegisterId,
    },
    PickOmit {
        kind: PickOmitOperation,
        destination: RegisterId,
        value: RegisterId,
        keys: Box<[usize]>,
    },
    CallGlobal0 {
        destination: RegisterId,
        slot: usize,
    },
    CallGlobal1 {
        destination: RegisterId,
        slot: usize,
        argument: RegisterId,
    },
    CallGlobal1FromGlobal {
        destination: RegisterId,
        slot: usize,
        argument_slot: usize,
    },
    CallGlobal2 {
        destination: RegisterId,
        slot: usize,
        arguments: [RegisterId; 2],
    },
    CallGlobal3 {
        destination: RegisterId,
        slot: usize,
        arguments: [RegisterId; 3],
    },
    CallGlobal4 {
        destination: RegisterId,
        slot: usize,
        arguments: [RegisterId; 4],
    },
    Call {
        destination: RegisterId,
        target: CallTarget,
        arguments: Box<[RegisterId]>,
        spreads: Box<[usize]>,
    },
    Access {
        kind: AccessOperation,
        destination: RegisterId,
        value: RegisterId,
        key: AccessKey,
    },
    Slice {
        destination: RegisterId,
        value: RegisterId,
        start: Option<SliceBound>,
        end: Option<SliceBound>,
        exclusive: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnaryOperation {
    Pos,
    Neg,
    Not,
    Plus,
    Type,
    ToBoolean,
    ToNumber,
    ToString,
    IsBoolean,
    IsNumber,
    IsString,
    IsRecord,
    IsArray,
    Assign,
    Length,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOperation {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Aeq,
    Naeq,
    Same,
    Nsame,
    In,
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AssertOperation {
    Initialized,
    NonNil,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UpvalueOperation {
    Get,
    Set,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PickOmitOperation {
    Pick,
    Omit,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AccessOperation {
    Has,
    Get,
    Set,
}

#[derive(Debug, Clone)]
pub(crate) enum CallTarget {
    Global(usize),
    Register(RegisterId),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AccessKey {
    Constant(usize),
    Register(RegisterId),
    Index(i64),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SliceBound {
    Constant(i64),
    Register(RegisterId),
}

pub(super) fn block_may_capture_frame(body: &[Instruction]) -> bool {
    body.iter().any(|instruction| match &instruction.kind {
        InstructionKind::Function { .. } | InstructionKind::Module { .. } => true,
        InstructionKind::If {
            then_body,
            else_body,
            ..
        } => block_may_capture_frame(then_body) || block_may_capture_frame(else_body),
        InstructionKind::Loop { body, .. } => block_may_capture_frame(body),
        InstructionKind::Op(_) | InstructionKind::Record { .. } | InstructionKind::Array { .. } => {
            false
        }
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Condition {
    Truthy,
    Falsy,
    Initialized,
    Uninitialized,
    Nil,
    NonNil,
}

#[derive(Debug, Clone)]
pub(crate) enum LoopKind {
    Infinite,
    Iterable {
        value: RegisterId,
    },
    Range {
        start: RegisterId,
        end: RegisterId,
        exclusive: bool,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum RecordKey {
    Constant(String),
    Dynamic(RegisterId),
    Index(i64),
}

#[derive(Debug, Clone)]
pub(crate) enum RecordElement {
    Field {
        key: RecordKey,
        value: RegisterId,
        optional: bool,
    },
    Spread(RegisterId),
}

#[derive(Debug, Clone)]
pub(crate) enum ArrayElement {
    Item(RegisterId),
    Range {
        start: RangeEndpoint,
        end: RangeEndpoint,
        exclusive: bool,
    },
    Spread(RegisterId),
}

#[derive(Debug, Clone)]
pub(crate) enum RangeEndpoint {
    Constant(i64),
    Dynamic(RegisterId),
}
