//! Conservative, intraprocedural reachability analysis. This pass never changes the AST or bytecode.

mod analyzer;
mod flow;

use crate::{diagnostic::DiagnosticsCollector, parser::Script};

use analyzer::Analyzer;
use flow::Flow;

pub(crate) fn analyze(script: &Script<'_>, diagnostics: &mut DiagnosticsCollector<'_, '_>) {
    Analyzer::new(diagnostics).block(&script.0, script.1.as_deref(), false);
}
