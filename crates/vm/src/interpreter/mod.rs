mod call;
mod call_stack;
mod control;
mod frame;
mod globals;
mod operation;
mod register;
mod runtime;
mod script_reference;

use std::rc::Rc;

use crate::bytecode::{
    AccessKey, AccessOperation, ArrayElement, AssertOperation, BinaryOperation, CallTarget,
    Condition, Instruction, InstructionKind, LoopKind, NumericOperation, Operation,
    PickOmitOperation, RangeEndpoint, RecordElement, RecordKey, SliceBound, UnaryOperation,
    UpvalueOperation,
};
use crate::{MiraError, MiraFunction, MiraModule, MiraValue, Result, RuntimeErrorKind, operations};

pub(crate) use call::ScriptFunction;
use call_stack::CallStack;
pub(crate) use control::ScriptModule;
pub(crate) use frame::FrameId;
use frame::{Frame, FrameArena};
pub(crate) use globals::{Globals, std_slot};
pub(crate) use register::RegisterId;
pub(crate) use runtime::ExecutionId;
pub use runtime::Runtime;

#[derive(Debug)]
enum Flow {
    Continue,
    Break,
    LoopContinue,
    Return(MiraValue),
}
