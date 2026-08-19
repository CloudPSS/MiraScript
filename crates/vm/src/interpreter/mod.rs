mod call;
mod control;
mod operation;
mod runtime;
mod state;

use indexmap::IndexMap;
use std::rc::Rc;

use crate::bytecode::{
    AccessKey, AccessOperation, ArrayElement, AssertOperation, BinaryOperation, CallTarget,
    Condition, FunctionDef, Instruction, InstructionKind, LoopKind, NumericOperation, Operation,
    PickOmitOperation, Program, RangeEndpoint, RecordElement, RecordKey, SliceBound,
    UnaryOperation, UpvalueOperation,
};
use crate::value::{MiraCallContext, NativeRuntime, ScriptModule};
use crate::{
    MiraAny, MiraContext, MiraError, MiraFunction, MiraModule, Result, RunOptions, operations,
};

pub(crate) use runtime::ExecutionId;
pub use runtime::Runtime;

const INLINE_CALL_DEPTH: usize = 8;
const INLINE_GLOBAL_SLOTS: usize = 8;

use state::{CallStack, Frame, FrameArena, GlobalSlots};

#[derive(Debug)]
enum Flow {
    Continue,
    Break,
    LoopContinue,
    Return(MiraAny),
}

pub(crate) fn run(
    program: &Program,
    context: &MiraContext,
    options: &RunOptions,
) -> Result<MiraAny> {
    let mut runtime = Runtime::new(program, context, options);
    let result = match runtime.execute_block(&program.root.body, 0)? {
        Flow::Return(value) => value,
        Flow::Continue => MiraAny::Nil,
        Flow::Break | Flow::LoopContinue => {
            return Err(MiraError::runtime("invalid root control flow"));
        }
    };
    if result.contains_script_reference(runtime.execution) {
        return Err(MiraError::EscapingClosure.into());
    }
    Ok(result)
}
