use super::*;

fn chunk(code: &[u8], constants: &[u8]) -> Vec<u8> {
    let length = 4 + code.len() + 4 + constants.len();
    let mut chunk = Vec::with_capacity(length + 4);
    chunk.extend_from_slice(&(length as u32).to_le_bytes());
    chunk.extend_from_slice(&(code.len() as u32).to_le_bytes());
    chunk.extend_from_slice(code);
    chunk.extend_from_slice(&(constants.len() as u32).to_le_bytes());
    chunk.extend_from_slice(constants);
    chunk
}

fn root(body: &[u8], register_count: u8) -> Vec<u8> {
    let mut code = vec![OpCode::Func as u8, 0, 0, register_count];
    code.extend_from_slice(body);
    code.push(OpCode::FuncEnd as u8);
    code
}

#[test]
fn rejects_truncated_header() {
    let error = Program::decode(&[0; 3]).unwrap_err();
    assert!(matches!(error, MiraError::InvalidBytecode { .. }));
}

#[test]
fn decodes_compiler_output() {
    let (chunk, diagnostics) =
        mirascript_core::Compiler::compile("1 + 2", &mirascript_core::Config::new());
    assert!(diagnostics.is_empty());
    Program::decode(&chunk.unwrap()).unwrap();
}

#[test]
fn decodes_every_constant_encoding() {
    let mut constants = vec![0, 1, 2, 3];
    constants.extend_from_slice(&(-7_i32).to_le_bytes());
    constants.push(4);
    constants.extend_from_slice(&1.25_f64.to_le_bytes());
    constants.push(5);
    constants.extend_from_slice(&1_u32.to_le_bytes());
    constants.push(b'x');
    let program = Program::decode(&chunk(&root(&[], 0), &constants)).unwrap();
    assert_eq!(
        program.constants.as_ref(),
        &[
            MiraAny::Nil,
            MiraAny::Boolean(true),
            MiraAny::Boolean(false),
            MiraAny::Number(-7.0),
            MiraAny::Number(1.25),
            MiraAny::String("x".into()),
        ],
    );
}

#[test]
fn decodes_wide_registers_and_constants() {
    let wide = OpCode::WIDE_MASK;
    let mut code = vec![OpCode::Func as u8 | wide];
    code.extend_from_slice(&0_u32.to_le_bytes());
    code.extend_from_slice(&0_u32.to_le_bytes());
    code.extend_from_slice(&300_u32.to_le_bytes());
    code.push(OpCode::Constant as u8 | wide);
    code.extend_from_slice(&300_u32.to_le_bytes());
    code.extend_from_slice(&0_u32.to_le_bytes());
    code.push(OpCode::Return as u8 | wide);
    code.extend_from_slice(&300_u32.to_le_bytes());
    code.push(OpCode::FuncEnd as u8);
    let program = Program::decode(&chunk(&code, &[0])).unwrap();
    assert_eq!(program.root.register_count, 300);
}

#[test]
fn rejects_malformed_constants() {
    for constants in [vec![99], vec![4, 0], vec![5, 1, 0, 0, 0, 0xff]] {
        assert!(matches!(
            Program::decode(&chunk(&root(&[], 0), &constants)),
            Err(MiraError::InvalidBytecode { .. })
        ));
    }
}

#[test]
fn rejects_unknown_truncated_and_out_of_range_instructions() {
    let cases = [
        root(&[0x7f], 0),
        root(&[OpCode::Constant as u8 | OpCode::WIDE_MASK, 0], 1),
        root(&[OpCode::Constant as u8, 1, 0], 1),
        root(&[OpCode::Uninit as u8, 1], 0),
    ];
    for code in cases {
        assert!(matches!(
            Program::decode(&chunk(&code, &[])),
            Err(MiraError::InvalidBytecode { .. })
        ));
    }
}

#[test]
fn rejects_illegal_nesting_and_wide_terminators() {
    for code in [
        root(&[OpCode::IfEnd as u8], 0),
        vec![
            OpCode::Func as u8,
            0,
            0,
            0,
            OpCode::FuncEnd as u8 | OpCode::WIDE_MASK,
        ],
    ] {
        assert!(matches!(
            Program::decode(&chunk(&code, &[])),
            Err(MiraError::InvalidBytecode { .. })
        ));
    }
}

#[test]
fn loop_frame_reuse_excludes_captured_environments() {
    fn first_loop(body: &[Instruction]) -> Option<bool> {
        body.iter().find_map(|instruction| match &instruction.kind {
            InstructionKind::Loop { reuse_frame, .. } => Some(*reuse_frame),
            InstructionKind::If {
                then_body,
                else_body,
                ..
            } => first_loop(then_body).or_else(|| first_loop(else_body)),
            _ => None,
        })
    }

    let decode = |source: &str| {
        let (chunk, diagnostics) =
            mirascript_core::Compiler::compile(source, &mirascript_core::Config::new());
        assert!(diagnostics.is_empty());
        Program::decode(&chunk.unwrap()).unwrap()
    };

    let scalar = decode("let mut total = 0; for value in 1..10 { total += value; } total");
    assert_eq!(first_loop(&scalar.root.body), Some(true));

    let captured =
        decode("let mut first = nil; for value in 1..2 { first = fn { value }; } first()");
    assert_eq!(first_loop(&captured.root.body), Some(false));
}

#[test]
fn static_calls_with_small_fixed_arity_are_quickened() {
    let (chunk, diagnostics) = mirascript_core::Compiler::compile(
        "f(); f(1); f(1, 2); f(1, 2, 3); f(1, 2, 3, 4); nil",
        &mirascript_core::Config::new(),
    );
    assert!(diagnostics.is_empty());
    let program = Program::decode(&chunk.unwrap()).unwrap();
    let arities: Vec<_> = program
        .root
        .body
        .iter()
        .filter_map(|instruction| match &instruction.kind {
            InstructionKind::Op(Operation::CallGlobal0 { .. }) => Some(0),
            InstructionKind::Op(Operation::CallGlobal1 { .. }) => Some(1),
            InstructionKind::Op(Operation::CallGlobal2 { .. }) => Some(2),
            InstructionKind::Op(Operation::CallGlobal3 { .. }) => Some(3),
            InstructionKind::Op(Operation::CallGlobal4 { .. }) => Some(4),
            _ => None,
        })
        .collect();
    assert_eq!(arities, [0, 1, 2, 3, 4]);
}

#[test]
fn arithmetic_uses_numeric_operations() {
    let (chunk, diagnostics) = mirascript_core::Compiler::compile(
        "[left + right, left - right, left * right, left / right, left % right, left ^ right]",
        &mirascript_core::Config::new(),
    );
    assert!(diagnostics.is_empty());
    let program = Program::decode(&chunk.unwrap()).unwrap();
    let operations: Vec<_> = program
        .root
        .body
        .iter()
        .filter_map(|instruction| match &instruction.kind {
            InstructionKind::Op(Operation::Numeric { kind, .. }) => Some(*kind),
            _ => None,
        })
        .collect();
    assert_eq!(
        operations,
        [
            NumericOperation::Add,
            NumericOperation::Sub,
            NumericOperation::Mul,
            NumericOperation::Div,
            NumericOperation::Mod,
            NumericOperation::Pow,
        ]
    );
}

#[test]
fn static_calls_borrow_adjacent_global_arguments() {
    let (chunk, diagnostics) =
        mirascript_core::Compiler::compile("function(argument)", &mirascript_core::Config::new());
    assert!(diagnostics.is_empty());
    let program = Program::decode(&chunk.unwrap()).unwrap();
    assert!(program.root.body.iter().any(|instruction| matches!(
        instruction.kind,
        InstructionKind::Op(Operation::CallGlobal1FromGlobal { .. })
    )));
}
