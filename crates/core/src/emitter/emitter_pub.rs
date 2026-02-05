use std::borrow::Cow;

use crate::{
    DiagnosticCode,
    diagnostic::DiagnosticsCollector,
    parser::{AstWalker, TokenRef},
};

use super::{Emitter, OpCode, opcode::Register, variable::Variable};

pub(super) type ModuleExports<'s, 'c> = Option<ModuleExportsData<'s, 'c>>;

type OnExport<'s, 'c> = Box<dyn FnOnce(&mut Emitter<'s, 'c>) + 's>;
pub(super) struct ModuleExportsData<'s, 'c> {
    module_id: TokenRef<'s>,
    exports: Vec<OnExport<'s, 'c>>,
}

impl<'s, 'c> ModuleExportsData<'s, 'c> {
    pub fn new(module_id: &TokenRef<'s>) -> ModuleExports<'s, 'c> {
        Some(ModuleExportsData {
            module_id: module_id.clone(),
            exports: vec![],
        })
    }

    pub fn commit(self, emitter: &mut Emitter<'s, 'c>, id: impl Into<Cow<'s, str>>, reg: Register) {
        let id_const = emitter.add_const_string(id);
        emitter.op_2(self.module_id.range(), OpCode::Module, reg, id_const);
        for e in self.exports {
            e(emitter);
        }
        emitter.op(self.module_id.range(), OpCode::Freeze);
    }
}

pub(super) fn emit_pub<'s, 'c>(
    diagnostics: &mut DiagnosticsCollector<'s, 'c>,
    kw_pub: &Option<TokenRef<'s>>,
    exports: &mut ModuleExports<'s, 'c>,
    variable: &mut Variable<'s>,
) {
    if let Some(kw_pub) = kw_pub {
        if let Some(exports) = exports {
            let range = kw_pub.range();
            let id = variable.name();
            let a = variable.register();
            variable.mark_exported(&exports.module_id);
            exports.exports.push(Box::new(move |emitter| {
                let id = emitter.add_const_string(id);
                emitter.op_2(range, OpCode::Field, id, a);
            }));
        } else {
            diagnostics.push(DiagnosticCode::UnexpectedPub, kw_pub.range());
        }
    }
}
