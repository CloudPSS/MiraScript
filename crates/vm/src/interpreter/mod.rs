mod call;
mod control;
mod operation;
mod runtime;
mod script_reference;
mod state;

use std::rc::Rc;

use crate::bytecode::{
    AccessKey, AccessOperation, ArrayElement, AssertOperation, BinaryOperation, CallTarget,
    Condition, FunctionDef, Instruction, InstructionKind, LoopKind, NumericOperation, Operation,
    PickOmitOperation, RangeEndpoint, RecordElement, RecordKey, SliceBound, UnaryOperation,
    UpvalueOperation,
};
use crate::{MiraError, MiraFunction, MiraModule, MiraValue, Result, RuntimeErrorKind, operations};

pub(crate) use call::ScriptFunction;
pub(crate) use control::ScriptModule;
pub(crate) use runtime::ExecutionId;
pub use runtime::Runtime;
pub(crate) use state::FrameId;
use state::{CallStack, Frame, FrameArena, MiraAny};

use self::state::ROOT_FRAME_ID;

const INLINE_CALL_DEPTH: usize = 8;

#[derive(Debug)]
enum Flow {
    Continue,
    Break,
    LoopContinue,
    Return(MiraValue),
}
