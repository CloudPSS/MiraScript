use super::constants::Constant;

#[derive(Debug)]
pub(crate) struct Program {
    pub constants: Box<[Constant]>,
    pub global_names: Box<[String]>,
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
        destination: usize,
        function: usize,
    },
    If {
        condition: Condition,
        register: usize,
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
        destination: usize,
        elements: Vec<RecordElement>,
    },
    Array {
        destination: usize,
        elements: Vec<ArrayElement>,
    },
    Module {
        destination: usize,
        name: String,
        fields: Vec<(String, usize)>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Noop,
    Break,
    Continue,
    Return {
        value: usize,
    },
    Constant {
        destination: usize,
        constant: usize,
    },
    Uninit {
        destination: usize,
    },
    Unary {
        kind: UnaryOperation,
        destination: usize,
        value: usize,
    },
    Numeric {
        kind: NumericOperation,
        destination: usize,
        left: usize,
        right: usize,
    },
    Binary {
        kind: BinaryOperation,
        destination: usize,
        left: usize,
        right: usize,
    },
    Swap {
        left: usize,
        right: usize,
    },
    Upvalue {
        kind: UpvalueOperation,
        value: usize,
        level: usize,
        register: usize,
    },
    GetGlobal {
        destination: usize,
        slot: usize,
    },
    GetGlobalDyn {
        destination: usize,
        key: usize,
    },
    InGlobal {
        destination: usize,
        key: usize,
    },
    Concat {
        destination: usize,
        values: Box<[usize]>,
    },
    Format {
        destination: usize,
        value: usize,
        format: usize,
    },
    Assert {
        kind: AssertOperation,
        value: usize,
    },
    PickOmit {
        kind: PickOmitOperation,
        destination: usize,
        value: usize,
        keys: Box<[usize]>,
    },
    CallGlobal0 {
        destination: usize,
        slot: usize,
    },
    CallGlobal1 {
        destination: usize,
        slot: usize,
        argument: usize,
    },
    CallGlobal1FromGlobal {
        destination: usize,
        slot: usize,
        argument_slot: usize,
    },
    CallGlobal2 {
        destination: usize,
        slot: usize,
        arguments: [usize; 2],
    },
    CallGlobal3 {
        destination: usize,
        slot: usize,
        arguments: [usize; 3],
    },
    CallGlobal4 {
        destination: usize,
        slot: usize,
        arguments: [usize; 4],
    },
    Call {
        destination: usize,
        target: CallTarget,
        arguments: Box<[usize]>,
        spreads: Box<[usize]>,
    },
    Access {
        kind: AccessOperation,
        destination: usize,
        value: usize,
        key: AccessKey,
    },
    Slice {
        destination: usize,
        value: usize,
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
pub(crate) enum NumericOperation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
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
    Register(usize),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum AccessKey {
    Constant(usize),
    Register(usize),
    Index(i64),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SliceBound {
    Constant(i64),
    Register(usize),
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
        value: usize,
    },
    Range {
        start: usize,
        end: usize,
        exclusive: bool,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum RecordKey {
    Constant(String),
    Dynamic(usize),
    Index(i64),
}

#[derive(Debug, Clone)]
pub(crate) enum RecordElement {
    Field {
        key: RecordKey,
        value: usize,
        optional: bool,
    },
    Spread(usize),
}

#[derive(Debug, Clone)]
pub(crate) enum ArrayElement {
    Item(usize),
    Range {
        start: RangeEndpoint,
        end: RangeEndpoint,
        exclusive: bool,
    },
    Spread(usize),
}

#[derive(Debug, Clone)]
pub(crate) enum RangeEndpoint {
    Constant(i64),
    Dynamic(usize),
}
