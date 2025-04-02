pub(super) fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch == '@' || unicode_ident::is_xid_start(ch)
}

pub(super) fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch == '$' || ch == '@' || unicode_ident::is_xid_continue(ch)
}
