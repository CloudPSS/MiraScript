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
    Add,
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
    /// LTE %ret %1 %2\
    /// %ret = %1 <= %2
    Lte,
    /// GT %ret %1 %2\
    /// %ret = %1 > %2
    Gt,
    /// GTE %ret %1 %2\
    /// %ret = %1 >= %2
    Gte,
    /// AEQ %ret %1 %2\
    /// %ret = %1 ~= %2
    Aeq,
    /// NAEQ %ret %1 %2\
    /// %ret = %1 !~ %2
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
    /// IN_GLOBAL %ret %1 %2\
    /// %ret = %1 in global
    InGlobal,
    /// CONCAT %ret `n` %1 %2 ... %n\
    /// %ret = %1 .. %2 .. ... .. %n
    Concat,
    /// AND %ret %1 %2\
    /// %ret = %1 && %2
    And,
    /// OR %ret %1 %2\
    /// %ret = %1 || %2
    Or,
    /// ASSERT_INIT %v\
    /// assert(%v != uninitialized)
    AssertInit,
    /// ASSERT_NON_NIL %v\
    /// assert(%v != nil)
    AssertNonNil,
    /// TYPE %ret %1\
    /// %ret = type(%1)
    Type,
    /// TO_BOOLEAN %ret %1\
    /// %ret = boolean(%1)
    ToBoolean,
    /// TO_NUMBER %ret %1\
    /// %ret = number(%1)
    ToNumber,
    /// TO_STRING %ret %1\
    /// %ret = string(%1)
    ToString,
    // Hint: use Eq/Same %ret %1 %0 for is nil
    /// IS_BOOLEAN %ret %1\
    /// %ret = type(%1) == "boolean"
    IsBoolean,
    /// IS_NUMBER %ret %1\
    /// %ret = type(%1) == "number"
    IsNumber,
    /// IS_STRING %ret %1\
    /// %ret = type(%1) == "string"
    IsString,
    /// IS_RECORD %ret %1\
    /// %ret = type(%1) == "record"
    IsRecord,
    /// IS_ARRAY %ret %1\
    /// %ret = type(%1) == "array"
    IsArray,

    // variable management
    /// CONSTANT %reg `index`\
    /// %reg = CONSTANTS\[index]
    Constant,
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
    Record,
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
    /// PICK %ret %var `n` `key_1` `key_2` ... `key_n`\
    /// %ret = %var pick keys CONSTANTS\[key_1], CONSTANTS\[key_2], ..., CONSTANTS\[key_n]
    Pick,
    /// OMIT %ret %var `n` `key_1` `key_2` ... `key_n`\
    /// %ret = %var omit keys CONSTANTS\[key_1], CONSTANTS\[key_2], ..., CONSTANTS\[key_n]
    Omit,
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
    /// HAS %ret %var `key`\
    /// %ret = initialized(%var\[CONSTANTS\[key]])
    Has,
    /// HAS_DYN %ret %var %key\
    /// %ret = initialized(%var\[%key])
    HasDyn,
    /// HAS_INDEX %ret %var `index`\
    /// %ret = initialized(%var\[index])
    HasIndex,
    /// GET %ret %var `key`\
    /// %ret = %var\[CONSTANTS\[key]] ?? nil
    Get,
    /// GET_DYN %ret %var %key\
    /// %ret = %var\[%key] ?? nil
    GetDyn,
    /// GET_INDEX %ret %var `index`\
    /// %ret = %var\[index] ?? nil
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
    /// LENGTH %ret %var\
    /// %ret = $Length(%var)
    Length,

    // control flow
    /// LOOP `regn`\
    /// loop { let %1, .. ,%regn;
    Loop,
    /// LOOP_FOR `regn` %iterable\
    /// for %1 in %iterable { let %2, .. ,%regn;
    LoopFor,
    /// LOOP_RANGE `regn` %start %end\
    /// for %1 in %start..%end { let %2, .. ,%regn;
    LoopRange,
    /// LOOP_RANGE_EXCLUSIVE `regn` %start %end\
    /// for %1 in %start..<%end { let %2, .. ,%regn;
    LoopRangeExclusive,
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
    /// CALL %ret `f` `argn` %1 %2 ... %argn `spread_n` `spread_arg_a` ...\
    /// %ret = GLOBAL[CONSTANTS[f]](%1, %2, ... , %argn);
    ///
    /// If spread_arg_a is present, arg at that index will be spread
    Call,
    /// CALL_DYN %ret %f `argn` %1 %2 ... %argn `spread_n` `spread_arg_a` ...\
    /// %ret = %f(%1, %2, ... , %argn);
    ///
    /// If spread_arg_a is present, arg at that index will be spread
    CallDyn,
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> u8 {
        op as u8
    }
}

impl OpCode {
    pub const WIDE_MASK: u8 = 0x80;
    pub const PARAM_MAX: u32 = u8::MAX as u32;
    pub const PARAM_WIDE_MAX: u32 = u32::MAX;

    pub fn code(&self) -> u8 {
        *self as u8
    }
    pub fn wide_code(&self) -> u8 {
        *self as u8 | Self::WIDE_MASK
    }
}

pub trait OpParamTrait {
    fn value(&self) -> u32;
    fn is_wide(&self) -> bool {
        self.value() > OpCode::PARAM_MAX
    }
    fn code(&self) -> u8 {
        debug_assert!(!self.is_wide());
        self.value() as u8
    }
    fn wide_code(&self) -> [u8; 4] {
        (self.value()).to_le_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(u32);

impl Register {
    pub fn new(index: usize) -> Self {
        debug_assert!(index != 0);
        Self(index as u32)
    }

    /// Write to this register will be ignored
    /// Reading from this register will always return `nil`
    pub const EMPTY: Self = Self(0);

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl OpParamTrait for Register {
    fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpParam(i32);

impl OpParam {
    pub fn new(value: i32) -> Self {
        value.into()
    }
}

impl OpParamTrait for OpParam {
    fn value(&self) -> u32 {
        self.0 as u32
    }
}

impl From<usize> for OpParam {
    fn from(param: usize) -> Self {
        Self(param as i32)
    }
}

impl From<i32> for OpParam {
    fn from(param: i32) -> Self {
        Self(param)
    }
}
