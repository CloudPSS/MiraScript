mod call;
mod control;
mod operation;
mod state;

use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

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
use indexmap::IndexMap;

static NEXT_EXECUTION_ID: AtomicU64 = AtomicU64::new(1);
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
    let execution = NEXT_EXECUTION_ID.fetch_add(1, AtomicOrdering::Relaxed);
    let mut runtime = Runtime {
        program,
        context,
        options,
        globals: GlobalSlots::new(&program.global_names, context),
        execution,
        started: Instant::now(),
        checkpoint_remaining: options.checkpoint_interval.max(1),
        call_depth: 0,
        frames: FrameArena::new(program.root.register_count),
        call_stack: CallStack::new(),
    };
    let result = match runtime.execute_block(&program.root.body, 0)? {
        Flow::Return(value) => value,
        Flow::Continue => MiraAny::Nil,
        Flow::Break | Flow::LoopContinue => {
            return Err(MiraError::runtime("invalid root control flow").into());
        }
    };
    if result.contains_script_reference(execution) {
        return Err(MiraError::EscapingClosure.into());
    }
    Ok(result)
}

struct Runtime<'a> {
    program: &'a Program,
    context: &'a MiraContext,
    options: &'a RunOptions,
    globals: GlobalSlots<'a>,
    execution: u64,
    started: Instant,
    checkpoint_remaining: u32,
    call_depth: u32,
    frames: FrameArena,
    call_stack: CallStack,
}
