use std::ops::{Deref, DerefMut};

use super::prelude::*;
pub(crate) trait AstWalker<'s> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>);

    fn range(&self) -> SourceRange;
}

impl<'s> AstWalker<'s> for Token<'s> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        // Parsing 阶段不会创建新的 TokenKind::InterpolatedString，无需处理
        let TokenKind::Unknown {
            mut errors,
            recovered,
        } = std::mem::replace(&mut self.kind, TokenKind::Empty)
        else {
            return;
        };
        collector.append(&mut errors);
        self.kind = *recovered;
    }
    fn range(&self) -> SourceRange {
        self.range.clone()
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Vec<E> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        for item in self.iter_mut() {
            item.collect_diagnostics(collector);
        }
    }
    fn range(&self) -> SourceRange {
        match self.as_slice() {
            [] => SourceRange {
                start: usize::MAX,
                end: usize::MIN,
            },
            [single] => single.range(),
            [first, .., last] => first.range().start..last.range().end,
        }
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Option<E> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        if let Some(item) = self {
            item.collect_diagnostics(collector);
        }
    }
    fn range(&self) -> SourceRange {
        if let Some(item) = self {
            item.range()
        } else {
            SourceRange {
                start: usize::MAX,
                end: usize::MIN,
            }
        }
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Box<E> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        self.deref_mut().collect_diagnostics(collector);
    }
    fn range(&self) -> SourceRange {
        self.deref().range()
    }
}
