use mirascript_core::{CompileConfig, Compiler, DiagnosticCode, InputMode, SourceRange};

fn analyze(
    source: &str,
    config: &CompileConfig,
) -> (Option<Vec<u8>>, Vec<SourceRange>, Vec<DiagnosticCode>) {
    let mut compiler = Compiler::new(source, config);
    let tokens = compiler.lex().expect("lexing failed");
    let script = compiler.parse(&tokens).expect("parsing failed");
    let bytecode = compiler.emit(&script);
    let dead = compiler
        .diagnostics_collector
        .iter()
        .filter(|d| d.error == DiagnosticCode::UnreachableCode)
        .map(|d| d.range.clone())
        .collect();
    let errors = compiler
        .diagnostics_collector
        .iter()
        .filter(|d| d.is_error())
        .map(|d| d.error)
        .collect();
    (bytecode, dead, errors)
}

/// Brackets in test inputs mark exact diagnostic ranges, including no nested duplicates.
fn check(marked: &str) {
    let mut source = String::new();
    let mut expected = Vec::new();
    let mut start = None;
    for ch in marked.chars() {
        match ch {
            '«' => {
                assert!(start.is_none());
                start = Some(source.len());
            }
            '»' => expected.push(start.take().unwrap()..source.len()),
            _ => source.push(ch),
        }
    }
    assert!(start.is_none());
    let (bytecode, mut actual, errors) = analyze(&source, &CompileConfig::default());
    assert!(errors.is_empty(), "{source}: {errors:?}");
    assert!(bytecode.is_some(), "{source}");
    actual.sort_by_key(|r| r.start);
    assert_eq!(actual, expected, "{source}");
}

#[test]
fn direct_jumps_and_block_tails() {
    check("return 1; «debug_print(2); 3»");
    check("loop { break; «debug_print(1); debug_print(2);» } 3");
}

#[test]
fn nested_blocks_and_branch_exits() {
    check("{ return; } «1»");
    check("let x = { return; }; «1»");
    check("if x { return; } else { return; } «1»");
    check("if x { return; } else if y { return; } else { return; } «1»");
    check("x ? { return; } : { return; }; «1»");
    check("if x { return; } 1");
    check("if x { return; } else { 2 } 1");
    check("loop { if x { break; } else { continue; } «debug_print(1);» } 2");
}

#[test]
fn match_coverage_and_guards() {
    check("match x { case _ { 1 } «case 2 { return; 3 }» «case _ { 4 }» }");
    check("match x { case (value) { value } «case _ { 1 }» }");
    check("match x { case _ and _ { 1 } «case _ { 2 }» }");
    check("match x { case 1 or _ { 1 } «case 2 { 2 }» }");
    check("match x { case _ if true { 1 } «case _ { 2 }» }");
    check("match x { case _ if false «{ return; 1 }» case _ { 2 } }");
    check("match x { case _ if ready { 1 } case _ { 2 } }");
    check("match x { case 1 { return; } case _ { return; } } «1»");
    check("match x { case 1 { return; } } 1");
    check("match x { case _ if ready { return; } } 1");
    check("match x {} 1");
    check("match x { case _ if { return; } «{ 1 }» «case _ { 2 }» } «3»");
}

#[test]
fn boolean_constants_and_short_circuiting() {
    check("if false «{ return; 1 }» else { 2 }");
    check("if true { 1 } else «{ 2 }»");
    check("if true { 1 } else «if ready { 2 } else { 3 }»");
    check("if !(true && (false || true)) «{ 1 }» else { 2 }");
    check("if not (true and false) { 1 } else «{ 2 }»");
    check("false ? «{ return; }» : 1");
    check("true ? 1 : «{ return; }»");
    check("false && «{ return; 1 }»; 2");
    check("true || «{ return; 1 }»; 2");
    check("false and «f()»; true or «g()»;");
    check("true && { return; }; «1»");
    check("false || { return; }; «1»");
    check("x && { return; }; 1");
    check("x || { return; }; 1");
    check("x ?? { return; }; 1");
    check("let flag = false; if flag { 1 } else { 2 }");
    check("const @FLAG = false; if @FLAG { 1 } else { 2 }");
    check("if 1 == 2 { 1 } else { 2 }");
    check("if x && false { 1 } else { 2 }");
}

#[test]
fn boolean_summaries_preserve_syntactic_constant_boundaries() {
    check("if (true && true)! { 1 } else { 2 }");
    check("if { true && true } { 1 } else { 2 }");
    check("if (true ? true : «true») { 1 } else { 2 }");
    check("if (true ?? false) { 1 } else { 2 }");
    check("if (true is _) { 1 } else { 2 }");
    check("if -(true && true) { 1 } else { 2 }");
    check("if !(false || (false && «f()»)) { 1 } else «{ 2 }»");
    check("match x { case _ if !(false || false) { 1 } «case _ { 2 }» }");
    check("while !(false || false) { break; } else «{ 1 }» 2");
    check("if (unknown && false) || true { 1 } else { 2 }");
}

#[test]
fn long_logical_chains_keep_the_same_reachability() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            for count in [32, 128, 512, 1024] {
                let and = vec!["true"; count].join(" && ");
                check(&format!("if {and} {{ 1 }} else «{{ 2 }}»"));
                let or = vec!["false"; count].join(" || ");
                check(&format!("if {or} «{{ 1 }}» else {{ 2 }}"));
                let unknown = vec!["flag"; count].join(" && ");
                check(&format!("if {unknown} {{ 1 }} else {{ 2 }}"));
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn loop_exit_targets_and_else() {
    check("loop { continue; «debug_print(1);» } «2»");
    check("loop {} «1»");
    check("loop { return; } «1»");
    check("while true {} «1»");
    check("while (true || «x») {} «1»");
    check("while false «{ return; 1; }» else { 2 } 3");
    check("while true { break; } else «{ 1 }» 2");
    check("loop { if false «{ break; }» } «1»");
    check("loop { loop { break; } } «1»");
    check("loop { let f = fn { loop { break; } }; } «1»");
    check("loop { break { return; }; } «1»");
    check("while ready { return; } 1");
    check("while ready { return; } else { return; } «1»");
    check("while ready { break; } else { return; } 1");
    check("for x in items { return; } 1");
    check("for x in items { return; } else { return; } «1»");
    check("for x in items { break; } else { return; } 1");
    check("while { break; } «{}» 1");
    check("for x in { return; } «{}» else «{ 1 }» «2»");
}

#[test]
fn functions_are_hoisted_and_have_separate_exits() {
    check("return f(); fn f() { return 1; «2» } «3»");
    check("return; «1;» fn f() { 2 } «3»");
    check("fn f() { return; «1» } 2");
    check("let f = fn { return; «1» }; 2");
    check("return; «let f = fn { return; 1 }; 2»");
    check("if false «{ fn f() { return; 1 } f(); }»");
    check("mod m { return; «pub let x = 1;» } «2»");
}

#[test]
fn expression_evaluation_order_and_optional_calls() {
    check("({ return; }) + «1»; «2»");
    check("[1, { return; }, «2», «3»]; «4»");
    check("[0..{ return; }, «2»]; «3»");
    check("(a: { return; }, b: «1»); «2»");
    check("(\"${ return; }\": «1»); «2»");
    check("\"${ return; }$«{ 1 }»\"; «2»");
    check("({ return; })[«1»]; «2»");
    check("f({ return; }, «1»); «2»");
    check("global.f({ return; }, «1»); «2»");
    check("obj.f({ return; }, «1»); 2");
    check("f!({ return; }); «2»");
    check("({ return; })(«1»); «2»");
    check("({ return; })::f(«1»); «2»");
    check("({ return; })::obj.f(«1»); 2");
    check("type({ return; }); «1»");
    check("let mut x = 1; x = { return; }; «2»");
    check("let mut x = true; x &&= { return; }; 2");
}

#[test]
fn invalid_jumps_do_not_hide_later_code() {
    for source in [
        "break; 1",
        "continue; 1",
        "loop { fn f() { break; 1 } break; }",
    ] {
        let (_, dead, errors) = analyze(source, &CompileConfig::default());
        assert!(dead.is_empty(), "{source}: {dead:?}");
        assert!(!errors.is_empty(), "{source}");
    }
}

#[test]
fn hint_configuration_preserves_bytecode_and_existing_numbers() {
    let source = "return 1; 2";
    let config = CompileConfig::default();
    let (enabled, dead, _) = analyze(source, &config);
    let (disabled, none, _) = analyze(
        source,
        &CompileConfig {
            diagnostic_hint: false,
            ..config
        },
    );
    assert_eq!(enabled, disabled);
    assert_eq!(dead, [10..11]);
    assert!(none.is_empty());
    assert!(DiagnosticCode::UnreachableCode.is_hint());
    assert_eq!(DiagnosticCode::UnreachableCode.code(), 4010);
    assert_eq!(DiagnosticCode::PreferUppercaseConstant.code(), 4009);
    assert_eq!(
        DiagnosticCode::UnreachableCode.message(),
        "此处的代码不可达"
    );
}

#[test]
fn template_and_unicode_ranges() {
    check("let 变量 = 1; return 变量; «\"不可达😀\"»");
    let source = "你好${ return; \"不可达😀\" }结束";
    let start = source.find('"').unwrap();
    let end = source.rfind('"').unwrap() + 1;
    let (bytecode, dead, errors) = analyze(
        source,
        &CompileConfig {
            input_mode: InputMode::Template,
            ..CompileConfig::default()
        },
    );
    assert!(bytecode.is_some());
    assert!(errors.is_empty());
    assert_eq!(dead, [start..end]);
}
