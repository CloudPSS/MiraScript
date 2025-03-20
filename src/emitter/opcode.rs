#[repr(u8)]
enum OpCode {
    // debugging
    /// No operation
    NOOP = 0x00,
    /// Halt the program
    HALT = 0x01,
    /// Interrupt the program
    INT = 0x02,

    // constatns
    /// Push number `-1` onto the stack
    MINUS_ONE = 0x0F,
    /// Push number `0` onto the stack
    ZERO,
    /// Push number `1` onto the stack
    ONE,
    /// Push number `2` onto the stack
    TWO,
    /// Push number `3` onto the stack
    THREE,
    /// Push number `4` onto the stack
    FOUR,
    /// Push number `5` onto the stack
    FIVE,
    /// Push `nil` onto the stack
    NIL,
    /// Push `true` onto the stack
    TRUE,
    /// Push `false` onto the stack
    FALSE,
    /// CONSTANT `u8`
    /// Push a constant onto the stack
    CONSTANT,
    /// CONSTANT `u24`
    /// Push a constant onto the stack
    CONSTANT_WIDE,

    // operators
    /// Add the top two values on the stack
    ADD,
    /// Subtract the top two values on the stack
    SUB,
    /// Multiply the top two values on the stack
    MUL,
    /// Divide the top two values on the stack
    DIV,
    /// Modulo the top two values on the stack
    MOD,
    /// Exponent the top two values on the stack
    EXP,
    /// Negate the top value on the stack
    NEG,
    /// Not the top value on the stack
    NOT,
    /// Unary plus the top value on the stack
    PLUS,
    /// Equal the top two values on the stack
    EQ,
    /// Not equal the top two values on the stack
    NE,
    /// Less than the top two values on the stack
    LT,
    /// Less than or equal the top two values on the stack
    LE,
    /// Greater than the top two values on the stack
    GT,
    /// Greater than or equal the top two values on the stack
    GE,

    // pop / push
    /// Pop the top value from the stack
    POP,
    /// POP_N `u8`
    /// Pop the top `n` values from the stack
    POP_N,
    /// Duplicate the top value on the stack
    DUP,
    /// DUP_N `u8`
    /// Duplicate the top `n`th value on the stack
    DUP_N,
    /// DUP_N_WIDE `u24`
    /// Duplicate the top `n`th value on the stack
    DUP_N_WIDE,
    /// Swap the top two values on the stack
    SWAP,
    /// SWAP_N `u8`
    /// Swap the top `n`th value and the top value on the stack
    SWAP_N,
    /// SWAP_N_WIDE `u24`
    /// Swap the top `n`th value and the top value on the stack
    SWAP_N_WIDE,
    /// INIT_LOCAL `n`
    /// Push `n` uninitialized values onto the stack
    INIT_LOCAL,
    /// INIT_LOCAL_WIDE `u24`
    /// Push `n` uninitialized values onto the stack
    INIT_LOCAL_WIDE,
    /// GET_LOCAL `n`
    /// Push the `n`th local value onto the stack
    GET_LOCAL,
    /// GET_LOCAL_WIDE `u24`
    /// Push the `n`th local value onto the stack
    GET_LOCAL_WIDE,
    /// SET_LOCAL `n`
    /// Pop the top value from the stack and set the `n`th local value
    SET_LOCAL,
    /// SET_LOCAL_WIDE `u24`
    /// Pop the top value from the stack and set the `n`th local value
    SET_LOCAL_WIDE,
    /// GET_GLOBAL `u8`
    /// Push the global value with the given name onto the stack
    GET_GLOBAL,
    /// GET_GLOBAL_WIDE `u24`
    /// Push the global value with the given name onto the stack
    GET_GLOBAL_WIDE,

    // closures
    /// CLOSURE `u8` (`u8` `u8`)*
    /// Create a new closure with the given function and upvalues
    CLOSURE,
    /// CLOSURE_WIDE `u24` (`u8` `u24`)*
    /// Create a new closure with the given function and upvalues
    CLOSURE_WIDE,
    /// GET_UPVALUE `u8`
    /// Push the upvalue with the given index onto the stack
    GET_UPVALUE,
    /// GET_UPVALUE_WIDE `u24`
    /// Push the upvalue with the given index onto the stack
    GET_UPVALUE_WIDE,
    /// SET_UPVALUE `u8`
    /// Pop the top value from the stack and set the upvalue with the given index
    SET_UPVALUE,
    /// SET_UPVALUE_WIDE `u24`
    /// Pop the top value from the stack and set the upvalue with the given index
    SET_UPVALUE_WIDE,
    /// Pop the upvalue from the stack and close the upvalue
    CLOSE_UPVALUE,
    /// CLOSE_UPVALUE_N `u8`
    /// Pop the top `n` values from the stack and close the upvalues
    CLOSE_UPVALUE_N,

    // objects
    /// Create a new editable tuple, pushing it onto the stack
    TUPLE,
    /// TUPLE_INIT `u8`
    /// Pop value from the stack and add it to the peek tuple, with the given key
    TUPLE_INIT,
    /// TUPLE_INIT_WIDE `u24`
    /// Pop value from the stack and add it to the peek tuple, with the given key
    TUPLE_INIT_WIDE,
    /// Pop tuple from the stack, copy all pairs to the peek tuple
    TUPLE_ASSIGN,
    /// Finish tuple construction
    TUPLE_FREEZE,
    /// Create a new editable array, pushing it onto the stack
    ARRAY,
    /// Pop value from the stack and add it to the peek array
    ARRAY_INIT,
    /// Pop array from the stack, copy all values to the peek array
    ARRAY_APPEND,
    /// Finish array construction
    ARRAY_FREEZE,
    /// GET_INDEX `u8`
    /// Get the value with the given index from the peek array or tuple
    GET_INDEX,
    /// GET_INDEX_WIDE `u24`
    /// Get the value with the given index from the peek array or tuple
    GET_INDEX_WIDE,
    /// GET_INDEX_LONG `u48`
    /// Get the value with the given index from the peek array or tuple
    GET_INDEX_LONG,
    /// GET `u8`
    /// Get the value with the given key from the peek array or tuple
    GET,
    /// GET_WIDE `u24`
    /// Get the value with the given key from the peek array or tuple
    GET_WIDE,
    /// GET_DYN
    /// Pop key and get the value from the peek array or tuple
    GET_DYN,

    // call
    /// Call the top value on the stack
    CALL_0,
    /// Call the second value on the stack with the top value on the stack
    CALL_1,
    /// Call the third value on the stack with the top two values on the stack
    CALL_2,
    /// Call the fourth value on the stack with the top three values on the stack
    CALL_3,
    /// Call the fifth value on the stack with the top four values on the stack
    CALL_4,
    /// Call the sixth value on the stack with the top five values on the stack
    CALL_5,
    /// Call the seventh value on the stack with the top six values on the stack
    CALL_6,
    /// Call the eighth value on the stack with the top seven values on the stack
    CALL_7,
    /// Call the ninth value on the stack with the top eight values on the stack
    CALL_8,
    /// CALL `u8`
    /// Call the `n`th value on the stack with the top `n-1` values on the stack
    CALL,
    /// CALL_WIDE `u24`
    /// Call the `n`th value on the stack with the top `n-1` values on the stack
    CALL_WIDE,
    /// CALL_VIRT_0 `u8`
    /// Call by name from constant pool with no arguments
    CALL_VIRT_0,
    /// CALL_VIRT_1 `u8`
    /// Call by name from constant pool with one argument
    CALL_VIRT_1,
    /// CALL_VIRT_2 `u8`
    /// Call by name from constant pool with two arguments
    CALL_VIRT_2,
    /// CALL_VIRT_3 `u8`
    /// Call by name from constant pool with three arguments
    CALL_VIRT_3,
    /// CALL_VIRT_4 `u8`
    /// Call by name from constant pool with four arguments
    CALL_VIRT_4,
    /// CALL_VIRT_5 `u8`
    /// Call by name from constant pool with five arguments
    CALL_VIRT_5,
    /// CALL_VIRT_6 `u8`
    /// Call by name from constant pool with six arguments
    CALL_VIRT_6,
    /// CALL_VIRT_7 `u8`
    /// Call by name from constant pool with seven arguments
    CALL_VIRT_7,
    /// CALL_VIRT_8 `u8`
    /// Call by name from constant pool with eight arguments
    CALL_VIRT_8,
    /// CALL_VIRT `u8` `u8`
    /// Call by name from constant pool with `n` arguments
    CALL_VIRT,
    /// CALL_VIRT_WIDE `u24` `u24`
    /// Call by name from constant pool with `n` arguments
    CALL_VIRT_WIDE,
    /// CALL_VIRT_DYN `u8`
    /// Call by name from stack with `n` arguments
    CALL_VIRT_DYN,
    /// CALL_VIRT_DYN_WIDE `u24`
    /// Call by name from stack with `n` arguments
    CALL_VIRT_DYN_WIDE,
    /// Return from the current function with the top value on the stack
    RETURN,
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> u8 {
        op as u8
    }
}
