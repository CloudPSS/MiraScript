use std::ops::{Deref, DerefMut};

use strum::{Display, VariantArray};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, VariantArray, Display)]
pub enum OpCode {
    // debugging
    /// No operation
    Noop = 0x00,

    // operators
    /// ADD %ret %1 %2\
    /// %ret = %1 + %2
    Add = 0x10,
    /// SUB %ret %1 %2\
    /// %ret = %1 - %2
    Sub,
    /// MUL %ret %1 %2\
    /// %ret = %1 * %2
    Mul,
    /// DIV %ret %1 %2\
    /// %ret = %1 / %2
    Div,
    /// MOD %ret %1 %2\
    /// %ret = %1 % %2
    Mod,
    /// EXP %ret %1 %2\
    /// %ret = %1 ^ %2
    Pow,
    /// POS %ret %1\
    /// %ret = +%1
    Pos,
    /// NEG %ret %1\
    /// %ret = -%1
    Neg,
    /// NOT %ret %1\
    /// %ret = !%1
    Not,
    /// PLUS %ret %1\
    /// %ret = +%1
    Plus,
    /// EQ %ret %1 %2\
    /// %ret = %1 == %2
    Eq,
    /// NEQ %ret %1 %2\
    /// %ret = %1 != %2
    Neq,
    /// LT %ret %1 %2\
    /// %ret = %1 < %2
    Lt,
    /// LEQ %ret %1 %2\
    /// %ret = %1 <= %2
    Leq,
    /// GT %ret %1 %2\
    /// %ret = %1 > %2
    Gt,
    /// GEQ %ret %1 %2\
    /// %ret = %1 >= %2
    Geq,
    /// AEQ %ret %1 %2\
    /// %ret = %1 ~= %2
    Aeq,
    /// NAEQ %ret %1 %2\
    /// %ret = %1 !~= %2
    Naeq,
    /// SAME %ret %1 %2\
    /// %ret = %1 === %2 // Same value zero
    Same,
    /// Nsame %ret %1 %2\
    /// %ret = %1 !== %2
    Nsame,
    /// IN %ret %1 %2\
    /// %ret = %1 in %2
    In,
    /// CONCAT %ret `n` %1 %2 ... %n\
    /// %ret = %1 .. %2 .. ... .. %n
    Concat,
    // /// AND %ret %1 %2\
    // /// %ret = %1 && %2
    // And,
    // /// OR %ret %1 %2\
    // /// %ret = %1 || %2
    // Or,
    /// INIT %v\
    /// assert(%v != uninitialized)
    Init,
    /// NON_NIL %v\
    /// assert(%v != nil)
    NonNil,
    /// TYPE %ret %1\
    /// %ret = type(%1)
    Type,
    /// TO_BOOL %ret %1\
    /// %ret = bool(%1)
    ToBool,
    /// TO_NUMBER %ret %1\
    /// %ret = number(%1)
    ToNumber,
    /// TO_STRING %ret %1\
    /// %ret = string(%1)
    ToString,

    // variable management
    /// CONSTANT %reg `index`\
    /// %reg = CONSTANTS\[index]
    Constant = 0x30,
    /// UNINIT %reg
    /// %reg = uninitialized
    Uninit,
    /// ASSIGN %ret %1\
    /// %ret = %1
    Assign,
    /// SWAP %1 %2\
    /// (%1, %2) = (%2, %1)
    Swap,
    /// GET_UPVALUE %ret `level` %%up\
    /// %ret = UPVALUES\[level]\[%%up]
    GetUpvalue,
    /// SET_UPVALUE %value `level` %%up \
    /// UPVALUES\[level]\[%%up] = %value
    SetUpvalue,
    /// GET_GLOBAL %ret `name` \
    /// %ret = GLOBALS\[CONSTANTS\[name]]
    GetGlobal,
    /// GET_GLOBAL_DYN %ret %name \
    /// %ret = GLOBALS\[%name]
    GetGlobalDyn,

    // objects
    /// RECORD %ret\
    /// %ret = (
    Record = 0x40,
    /// FIELD `name` %field\
    /// \[CONSTANTS\[name]]: %field,
    Field,
    /// FIELD_DYN %name %field\
    /// \[%name]: %field,
    FieldDyn,
    /// FIELD_INDEX `index` %field\
    /// \[index]: %field,
    FieldIndex,
    /// FIELD_OPT `name` %field\
    /// \[CONSTANTS\[name]]?: %field,
    FieldOpt,
    /// FIELD_OPT_DYN %name %field\
    /// \[%name]?: %field,
    FieldOptDyn,
    /// FIELD_OPT_INDEX `index` %field\
    /// \[index]?: %field,
    FieldOptIndex,
    /// ARRAY %ret\
    /// %ret = \[
    Array,
    /// ITEM %item\
    /// %item,
    Item,
    /// ITEM_RANGE `start` `end`\
    /// `start`..`end`,
    ItemRange,
    /// ITEM_RANGE_DYN %start %end\
    /// %start..%end,
    ItemRangeDyn,
    /// ITEM_RANGE_EXCLUSIVE_DYN %start %end\
    /// %start..<%end,
    ItemRangeExclusiveDyn,
    /// SPREAD %var\
    /// ..%var,
    Spread,
    /// FREEZE\
    /// ) for record or ] for array
    Freeze,
    /// GET %ret %var `key`\
    /// %ret = %var\[CONSTANTS\[key]]
    Get,
    /// GET_DYN %ret %var %key\
    /// %ret = %var\[%key]
    GetDyn,
    /// GET_INDEX %ret %var `index`\
    /// %ret = %var\[index]
    GetIndex,
    /// SET %value %var `key`\
    /// %var\[CONSTANTS\[key]] = %value
    Set,
    /// SET_DYN %value %var %key\
    /// %var\[%key] = %value
    SetDyn,
    /// SET_INDEX %value %var `index`\
    /// %var\[index] = %value
    SetIndex,
    /// SLICE %ret %var `start` `end`\
    /// %ret = %var\[start..end]
    Slice,
    /// SLICE_START %ret %var `end`\
    /// %ret = %var\[..end]
    SliceStart,
    /// SLICE_END %ret %var `start`\
    /// %ret = %var\[start..]
    SliceEnd,
    /// SLICE_DYN %ret %var %start %end\
    /// %ret = %var\[%start..%end]
    SliceDyn,
    /// SLICE_EXCLUSIVE_DYN %ret %var %start %end\
    /// %ret = %var\[%start..<%end]
    SliceExclusiveDyn,

    // control flow
    /// LOOP\
    /// loop {
    Loop = 0x60,
    /// LOOP_FOR %iterator %iterable\
    /// for %iterator in %iterable {
    LoopFor,
    /// LOOP_END\
    /// }
    LoopEnd,
    /// BREAK\
    /// break;
    Break,
    /// CONTINUE\
    /// continue;
    Continue,
    /// IF %cond\
    /// if (%cond) {
    If,
    /// IF_NOT %cond\
    /// if (!%cond) {
    IfNot,
    /// IF_INIT %var\
    /// if (initialized(%var)) {
    IfInit,
    /// IF_NOT_INIT %var\
    /// if (!initialized(%var)) {
    IfNotInit,
    /// IF_NIL %var\
    /// if (%var == nil) {
    IfNil,
    /// IF_NOT_NIL %var\
    /// if (%var != nil) {
    IfNotNil,
    /// ELSE\
    /// } else {
    Else,
    /// EL_IF IF*\
    /// } else if *** {\
    /// This instruction must be followed by an `IF*` instruction\
    ElIf,
    /// IF_END\
    /// }
    IfEnd,
    /// FUNC %f `argn` `regn`\
    /// %f = (%1, %2, ... , %argn) => { let %argn+1, ... , %regn;
    Func,
    /// FUNC_VARG %f `argn` `regn`\
    /// %f = (%1, %2, ... , %argn-1, ...%argn) => { let %argn+1, ... , %regn;
    FuncVarg,
    /// FUNC_END\
    /// }
    FuncEnd,
    /// RETURN %value\
    /// return %value;
    Return,
    /// CALL %ret `f` `argn` %1 %2 ... %argn\
    /// %ret = GLOBAL[CONSTANTS[f]](%1, %2, ... , %argn);
    Call,
    /// CALL_DYN %ret %f `argn` %1 %2 ... %argn\
    /// %ret = %f(%1, %2, ... , %argn);
    CallDyn,
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> u8 {
        op as u8
    }
}

impl OpCode {
    pub const WIDE_MASK: u8 = 0x80;
    pub const PARAM_MAX: usize = u8::MAX as usize;
    pub const PARAM_WIDE_MAX: usize = u32::MAX as usize;

    pub fn code(&self) -> u8 {
        *self as u8
    }
    pub fn wide_code(&self) -> u8 {
        *self as u8 | Self::WIDE_MASK
    }
}

pub trait OpParamTrait {
    fn value(&self) -> usize;
    fn is_wide(&self) -> bool {
        self.value() > OpCode::PARAM_MAX
    }
    fn code(&self) -> u8 {
        debug_assert!(!self.is_wide());
        self.value() as u8
    }
    fn wide_code(&self) -> [u8; 4] {
        (self.value() as u32).to_le_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(usize);

impl Register {
    pub fn new(index: usize) -> Self {
        debug_assert!(index != 0);
        Self(index)
    }

    /// Write to this register will be ignored
    /// Reading from this register will always return `nil`
    pub const EMPTY: Self = Self(0);

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl OpParamTrait for Register {
    fn value(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpParam(usize);

impl OpParam {
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}

impl OpParamTrait for OpParam {
    fn value(&self) -> usize {
        self.0
    }
}

impl From<usize> for OpParam {
    fn from(param: usize) -> Self {
        Self(param)
    }
}
