#[repr(u8)]
enum OpCode {
    // debugging
    /// No operation
    Noop = 0x00,

    // constatns
    /// Push number `-1` onto the stack
    MinusOne = 0x0F,
    /// Push number `0` onto the stack
    Zero,
    /// Push number `1` onto the stack
    One,
    /// Push number `2` onto the stack
    Two,
    /// Push number `3` onto the stack
    Three,
    /// Push number `4` onto the stack
    Four,
    /// Push number `5` onto the stack
    Five,
    /// Push `nil` onto the stack
    Nil,
    /// Push `true` onto the stack
    True,
    /// Push `false` onto the stack
    False,
    /// CONSTANT `u8`
    /// Push a constant onto the stack
    Constant,
    /// CONSTANT `u24`
    /// Push a constant onto the stack
    ConstantWide,

    // operators
    /// Add the top two values on the stack
    Add = 0x20,
    /// Subtract the top two values on the stack
    Sub,
    /// Multiply the top two values on the stack
    Mul,
    /// Divide the top two values on the stack
    Div,
    /// Modulo the top two values on the stack
    Mod,
    /// Exponent the top two values on the stack
    Exp,
    /// Negate the top value on the stack
    Neg,
    /// Not the top value on the stack
    Not,
    /// Unary plus the top value on the stack
    Plus,
    /// Equal the top two values on the stack
    Eq,
    /// Not equal the top two values on the stack
    Neq,
    /// Less than the top two values on the stack
    Lt,
    /// Less than or equal the top two values on the stack
    Leq,
    /// Greater than the top two values on the stack
    Gt,
    /// Greater than or equal the top two values on the stack
    Geq,

    // pop / push
    /// Pop the top value from the stack
    Pop = 0x40,
    /// POP_N `u8`
    /// Pop the top `n` values from the stack
    PopN,
    /// Duplicate the top value on the stack
    Dup,
    /// DUP_N `u8`
    /// Duplicate the top `n`th value on the stack
    DupN,
    /// DUP_N_WIDE `u24`
    /// Duplicate the top `n`th value on the stack
    DupNWide,
    /// Swap the top two values on the stack
    Swap,
    /// SWAP_N `u8`
    /// Swap the top `n`th value and the top value on the stack
    SwapN,
    /// SWAP_N_WIDE `u24`
    /// Swap the top `n`th value and the top value on the stack
    SwapNWide,
    /// INIT_LOCAL `n`
    /// Push `n` uninitialized values onto the stack
    InitLocal,
    /// INIT_LOCAL_WIDE `u24`
    /// Push `n` uninitialized values onto the stack
    InitLocalWide,
    /// GET_LOCAL `n`
    /// Push the `n`th local value onto the stack
    GetLocal,
    /// GET_LOCAL_WIDE `u24`
    /// Push the `n`th local value onto the stack
    GetLocalWide,
    /// SET_LOCAL `n`
    /// Pop the top value from the stack and set the `n`th local value
    SetLocal,
    /// SET_LOCAL_WIDE `u24`
    /// Pop the top value from the stack and set the `n`th local value
    SetLocalWide,
    /// GET_GLOBAL `u8`
    /// Push the global value with the given name onto the stack
    GetGlobal,
    /// GET_GLOBAL_WIDE `u24`
    /// Push the global value with the given name onto the stack
    GetGlobalWide,
    /// GET_GLOBAL_DYN
    /// Pop the top value from the stack and push the global value with the given name onto the stack
    GetGlobalDyn,
    /// SET_GLOBAL `u8`
    /// Pop the top value from the stack and set the global value with the given name
    SetGlobal,
    /// SET_GLOBAL_WIDE `u24`
    /// Pop the top value from the stack and set the global value with the given name
    SetGlobalWide,
    /// SET_GLOBAL_DYN
    /// Pop the top two values from the stack and set the global value with the given name
    SetGlobalDyn,

    // closures
    /// CLOSURE `u8` (`u8` `u8`)*
    /// Create a new closure with the given function and upvalues
    Closure = 0x60,
    /// CLOSURE_WIDE `u24` (`u8` `u24`)*
    /// Create a new closure with the given function and upvalues
    ClosureWide,
    /// GET_UPVALUE `u8`
    /// Push the upvalue with the given index onto the stack
    GetUpvalue,
    /// GET_UPVALUE_WIDE `u24`
    /// Push the upvalue with the given index onto the stack
    GetUpvalueWide,
    /// SET_UPVALUE `u8`
    /// Pop the top value from the stack and set the upvalue with the given index
    SetUpvalue,
    /// SET_UPVALUE_WIDE `u24`
    /// Pop the top value from the stack and set the upvalue with the given index
    SetUpvalueWide,
    /// Pop the upvalue from the stack and close the upvalue
    CloseUpvalue,
    /// CLOSE_UPVALUE_N `u8`
    /// Pop the top `n` values from the stack and close the upvalues
    CloseUpvalueN,

    // objects
    /// Create a new editable record, pushing it onto the stack
    Record = 0x70,
    /// RECORD_INIT `u8`
    /// Pop value from the stack and add it to the peek record, with the given key
    RecordInit,
    /// RECORD_INIT_WIDE `u24`
    /// Pop value from the stack and add it to the peek record, with the given key
    RecordInitWide,
    /// Pop record from the stack, copy all pairs to the peek record
    RecordAssign,
    /// Finish record construction
    RecordFreeze,
    /// Create a new editable array, pushing it onto the stack
    Array,
    /// Pop value from the stack and add it to the peek array
    ArrayInit,
    /// Pop array from the stack, copy all values to the peek array
    ArrayAppend,
    /// Finish array construction
    ArrayFreeze,
    /// GET_INDEX `u8`
    /// Get the value with the given index from the peek array or record
    GetIndex,
    /// GET_INDEX_WIDE `u24`
    /// Get the value with the given index from the peek array or record
    GetIndexWide,
    /// GET_INDEX_INT `u32`
    /// Get the value with the given index from the peek array or record
    GetIndexInt,
    /// GET `u8`
    /// Get the value with the given key from the peek array or record
    Get,
    /// GET_WIDE `u24`
    /// Get the value with the given key from the peek array or record
    GetWide,
    /// GET_DYN
    /// Pop key and get the value from the peek array or record
    GetDyn,
    /// SET `u8`
    /// Pop value and set the value in the peek external object
    Set,
    /// SET_WIDE `u24`
    /// Pop value and set the value in the peek external object
    SetWide,
    /// SET_DYN
    /// Pop key and value and set the value in the peek external object
    SetDyn,

    // control flow
    /// JUMP `u8`
    /// Jump to the given offset
    Jump = 0x90,
    /// JUMP_WIDE `u24`
    /// Jump to the given offset
    JumpWide,
    /// JUMP_IF_TRUE `u8`
    /// Pop the top value from the stack and jump to the given offset if it is truthy
    JumpIfTrue,
    /// JUMP_IF_TRUE_WIDE `u24`
    /// Pop the top value from the stack and jump to the given offset if it is truthy
    JumpIfTrueWide,
    /// JUMP_IF_FALSE `u8`
    /// Pop the top value from the stack and jump to the given offset if it is falsy
    JumpIfFalse,
    /// JUMP_IF_FALSE_WIDE `u24`
    /// Pop the top value from the stack and jump to the given offset if it is falsy
    JumpIfFalseWide,
    /// LOOP `u8`
    /// Jump to the given negative offset
    Loop,
    /// LOOP_WIDE `u24`
    /// Jump to the given negative offset
    LoopWide,
    /// LOOP_IF_TRUE `u8`
    /// Pop the top value from the stack and jump to the given negative offset if it is truthy
    LoopIfTrue,
    /// LOOP_IF_TRUE_WIDE `u24`
    /// Pop the top value from the stack and jump to the given negative offset if it is truthy
    LoopIfTrueWide,
    /// LOOP_IF_FALSE `u8`
    /// Pop the top value from the stack and jump to the given negative offset if it is falsy
    LoopIfFalse,
    /// LOOP_IF_FALSE_WIDE `u24`
    /// Pop the top value from the stack and jump to the given negative offset if it is falsy
    LoopIfFalseWide,

    // call
    /// Call the top value on the stack
    Call0 = 0xA0,
    /// Call the second value on the stack with the top value on the stack
    Call1,
    /// Call the third value on the stack with the top two values on the stack
    Call2,
    /// Call the fourth value on the stack with the top three values on the stack
    Call3,
    /// Call the fifth value on the stack with the top four values on the stack
    Call4,
    /// Call the sixth value on the stack with the top five values on the stack
    Call5,
    /// Call the seventh value on the stack with the top six values on the stack
    Call6,
    /// Call the eighth value on the stack with the top seven values on the stack
    Call7,
    /// Call the ninth value on the stack with the top eight values on the stack
    Call8,
    /// CALL `u8`
    /// Call the `n`th value on the stack with the top `n-1` values on the stack
    Call,
    /// CALL_WIDE `u24`
    /// Call the `n`th value on the stack with the top `n-1` values on the stack
    CallWide,
    /// CALL_VIRT_0 `u8`
    /// Call by name from constant pool with no arguments
    CallVirt0 = 0xB0,
    /// CALL_VIRT_1 `u8`
    /// Call by name from constant pool with one argument
    CallVirt1,
    /// CALL_VIRT_2 `u8`
    /// Call by name from constant pool with two arguments
    CallVirt2,
    /// CALL_VIRT_3 `u8`
    /// Call by name from constant pool with three arguments
    CallVirt3,
    /// CALL_VIRT_4 `u8`
    /// Call by name from constant pool with four arguments
    CallVirt4,
    /// CALL_VIRT_5 `u8`
    /// Call by name from constant pool with five arguments
    CallVirt5,
    /// CALL_VIRT_6 `u8`
    /// Call by name from constant pool with six arguments
    CallVirt6,
    /// CALL_VIRT_7 `u8`
    /// Call by name from constant pool with seven arguments
    CallVirt7,
    /// CALL_VIRT_8 `u8`
    /// Call by name from constant pool with eight arguments
    CallVirt8,
    /// CALL_VIRT `u8` `u8`
    /// Call by name from constant pool with `n` arguments
    CallVirt,
    /// CALL_VIRT_WIDE `u24` `u24`
    /// Call by name from constant pool with `n` arguments
    CallVirtWide,
    /// CALL_VIRT_DYN `u8`
    /// Call by name from stack with `n` arguments
    CallVirtDyn,
    /// CALL_VIRT_DYN_WIDE `u24`
    /// Call by name from stack with `n` arguments
    CallVirtDynWide,
    /// Return from the current function with the top value on the stack
    Return = 0xC0,
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> u8 {
        op as u8
    }
}
