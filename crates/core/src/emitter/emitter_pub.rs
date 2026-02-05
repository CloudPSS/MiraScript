use std::borrow::Cow;

use crate::{
    DiagnosticCode,
    parser::{AstWalker, TokenRef},
};

use super::{Emitter, OpCode, opcode::Register};

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

impl<'s, 'c> Emitter<'s, 'c> {
    pub(super) fn declare_pub(
        &mut self,
        exports: &mut ModuleExports<'s, 'c>,
        kw_pub: &Option<TokenRef<'s>>,
        variable_name: &'s str,
    ) {
        let Some(kw_pub) = kw_pub else {
            return;
        };
        let Some(exports) = exports else {
            self.diagnostics
                .push(DiagnosticCode::UnexpectedPub, kw_pub.range());
            return;
        };
        let range = kw_pub.range();
        let variable = self
            .scopes
            .find_local_variable(variable_name)
            .unwrap_or_else(|| {
                panic!("Variable '{variable_name}' not found in current scope when declaring pub");
            });
        let reg = variable.register();
        variable.mark_exported(&exports.module_id);
        exports.exports.push(Box::new(move |emitter| {
            let id = emitter.add_const_string(variable_name);
            emitter.op_2(range, OpCode::Field, id, reg);
        }));
    }
}
