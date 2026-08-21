mod call;
mod control;
mod frame;
mod globals;
mod operation;
mod register;
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
use frame::Frame;
pub(crate) use frame::FrameId;
pub(crate) use globals::{Globals, std_slot};
pub(crate) use register::RegisterId;
pub(crate) use runtime::ExecutionId;
pub use runtime::Runtime;
use state::{CallStack, FrameArena};

const INLINE_CALL_DEPTH: usize = 8;

#[derive(Debug)]
enum Flow {
    Continue,
    Break,
    LoopContinue,
    Return(MiraValue),
}
