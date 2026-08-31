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
    assert!(matches!(error.as_ref(), MiraError::InvalidBytecode { .. }));
}

#[test]
fn decodes_compiler_output() {
    let (chunk, diagnostics) =
        mirascript_core::Compiler::compile("1 + 2", &mirascript_core::CompileConfig::new());
    assert!(diagnostics.is_empty());
    Program::decode(&chunk.unwrap()).unwrap();
}

#[test]
fn integer_literal_array_ranges_use_constant_endpoints() {
    let (chunk, diagnostics) = mirascript_core::Compiler::compile(
        "[1.0..2e0, -1.0..<2, 1_000..0x3E9, +1..+2, -2147483648..2147483647]",
        &mirascript_core::CompileConfig::new(),
    );
    assert!(diagnostics.is_empty());
    let program = Program::decode(&chunk.unwrap()).unwrap();
    let Some(InstructionKind::Array { elements, .. }) = program
        .root
        .body
        .iter()
        .map(|instruction| &instruction.kind)
        .find(|instruction| matches!(instruction, InstructionKind::Array { .. }))
    else {
        panic!("expected array instruction");
    };

    let ranges: Vec<_> = elements
        .iter()
        .map(|element| match element {
            ArrayElement::Range {
                start: RangeEndpoint::Constant(start),
                end: RangeEndpoint::Constant(end),
                exclusive,
            } => (*start, *end, *exclusive),
            _ => panic!("expected constant array range"),
        })
        .collect();
    assert_eq!(
        ranges,
        [
            (1, 2, false),
            (-1, 1, false),
            (1_000, 1_001, false),
            (1, 2, false),
            (-2_147_483_648, 2_147_483_647, false),
        ]
    );
}

#[test]
fn non_integer_literal_array_ranges_keep_dynamic_endpoints() {
    let (chunk, diagnostics) = mirascript_core::Compiler::compile(
        "[1.1..<2, 1..2e-1, 1 + 0..2, 1..1 + 1, 0..<-2147483648]",
        &mirascript_core::CompileConfig::new(),
    );
    assert!(diagnostics.is_empty());
    let program = Program::decode(&chunk.unwrap()).unwrap();
    let Some(InstructionKind::Array { elements, .. }) = program
        .root
        .body
        .iter()
        .map(|instruction| &instruction.kind)
        .find(|instruction| matches!(instruction, InstructionKind::Array { .. }))
    else {
        panic!("expected array instruction");
    };

    assert!(elements.iter().all(|element| matches!(
        element,
        ArrayElement::Range {
            start: RangeEndpoint::Dynamic(_),
            end: RangeEndpoint::Dynamic(_),
            ..
        }
    )));
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
            Constant::Nil,
            Constant::True,
            Constant::False,
            Constant::Int(-7),
            Constant::Float(1.25),
            Constant::String("x".into()),
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
            Program::decode(&chunk(&root(&[], 0), &constants))
                .unwrap_err()
                .as_ref(),
            MiraError::InvalidBytecode { .. }
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
            Program::decode(&chunk(&code, &[])).unwrap_err().as_ref(),
            MiraError::InvalidBytecode { .. }
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
            Program::decode(&chunk(&code, &[])).unwrap_err().as_ref(),
            MiraError::InvalidBytecode { .. }
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
            mirascript_core::Compiler::compile(source, &mirascript_core::CompileConfig::new());
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
        &mirascript_core::CompileConfig::new(),
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
fn arithmetic_uses_specialized_numeric_operations() {
    let (chunk, diagnostics) = mirascript_core::Compiler::compile(
        "[left + right, left - right, left * right, left / right, left % right, left ^ right]",
        &mirascript_core::CompileConfig::new(),
    );
    assert!(diagnostics.is_empty());
    let program = Program::decode(&chunk.unwrap()).unwrap();
    let operations: Vec<_> = program
        .root
        .body
        .iter()
        .filter_map(|instruction| match &instruction.kind {
            InstructionKind::Op(Operation::Add { .. }) => Some("add"),
            InstructionKind::Op(Operation::Sub { .. }) => Some("sub"),
            InstructionKind::Op(Operation::Mul { .. }) => Some("mul"),
            InstructionKind::Op(Operation::Div { .. }) => Some("div"),
            InstructionKind::Op(Operation::Mod { .. }) => Some("mod"),
            InstructionKind::Op(Operation::Pow { .. }) => Some("pow"),
            _ => None,
        })
        .collect();
    assert_eq!(operations, ["add", "sub", "mul", "div", "mod", "pow"]);
}

#[test]
fn static_calls_borrow_adjacent_global_arguments() {
    let (chunk, diagnostics) = mirascript_core::Compiler::compile(
        "function(argument)",
        &mirascript_core::CompileConfig::new(),
    );
    assert!(diagnostics.is_empty());
    let program = Program::decode(&chunk.unwrap()).unwrap();
    assert!(program.root.body.iter().any(|instruction| matches!(
        instruction.kind,
        InstructionKind::Op(Operation::CallGlobal1FromGlobal { .. })
    )));
}
