use crate::{
    DiagnosticCode,
    diagnostic::DiagnosticsCollector,
    lexer::{Token, TokenKind},
};

pub(super) fn recover_token<'s>(
    mut t: Token<'s>,
    diagnostics_collector: &mut DiagnosticsCollector<'_, '_>,
) -> Option<Token<'s>> {
    match t.kind {
        TokenKind::Empty => None,
        TokenKind::Unknown {
            recovered: token,
            errors,
        } => {
            diagnostics_collector.extend(errors);
            t.kind = *token;
            recover_token(t, diagnostics_collector)
        }
        TokenKind::InterpolatedString(ref mut v, _) => {
            diagnostics_collector.push(DiagnosticCode::Interpolation, t.range.clone());
            for (_, tokens, _) in v.iter_mut() {
                *tokens = tokens
                    .iter_mut()
                    .filter_map(|t| recover_token(t.clone(), diagnostics_collector))
                    .collect::<Vec<_>>();
            }
            Some(t)
        }
        TokenKind::String(_, _) => {
            diagnostics_collector.push(DiagnosticCode::String, t.range.clone());
            Some(t)
        }
        _ => Some(t),
    }
}
