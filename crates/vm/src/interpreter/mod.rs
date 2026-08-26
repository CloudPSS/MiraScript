mod any;
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
    Condition, Instruction, InstructionKind, LoopKind, Operation, PickOmitOperation, RangeEndpoint,
    RecordElement, RecordKey, SliceBound, UnaryOperation, UpvalueOperation,
};
use crate::{
    MiraError, MiraFunction, MiraModule, MiraValue, MiraValueKind, Result, RuntimeErrorKind,
    operations,
};

use any::MiraAny;
pub(crate) use call::ScriptFunction;
use call_stack::CallStack;
pub(crate) use control::ScriptModule;
use frame::FrameArena;
pub(crate) use frame::FrameId;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime() {
        let mut runtime = Runtime::default();
        runtime.insert_fn("custom", |runtime: &mut Runtime, args: &[MiraValue]| {
            if args.is_empty() {
                anyhow::bail!("Expected at least one argument");
            }
            let mut sum = 0.0;
            for arg in args {
                sum += operations::to_number(runtime, *arg)?;
            }
            Ok(sum)
        });

        assert_eq!(
            runtime
                .eval("custom(1, 2, 3)")
                .unwrap()
                .as_number_unchecked(),
            6.0
        );
        assert_eq!(
            runtime
                .eval_unchecked("custom(10, 20)")
                .as_number_unchecked(),
            30.0
        );
        assert!(matches!(
            runtime.eval("custom()").unwrap_err().as_ref(),
            MiraError::External(e) if e.to_string() == "Expected at least one argument"
        ));
    }
}
