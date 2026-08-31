use std::ops::Deref;

use crate::{Operator, parser::ListItem};

use super::prelude::*;

const MINIMUM_INLINE_WIDTH: usize = 20;

impl Formatter<'_> {
    pub fn list_items<T>(
        &self,
        items: &[ListItem<'_, T>],
        boundary: FormatDoc,
        force_tail_comma: bool,
    ) -> FormatDoc
    where
        T: Formattable,
    {
        if items.is_empty() {
            return Self::nil();
        }
        let last = items.len() - 1;
        let rigid_items = items.iter().enumerate().map(|(index, item)| {
            let item_doc = item.deref().format(self);
            if index < last || force_tail_comma {
                item_doc.append(self.token_or(item.tail_comma(), Operator::Comma))
            } else {
                item_doc
            }
        });
        let rigid_body = self.join(rigid_items, self.space());
        let docs = items.iter().enumerate().map(|(index, item)| {
            let item_doc = item.deref().format(self);
            if index == last {
                let comma = self.token_or(item.tail_comma(), Operator::Comma);
                if force_tail_comma {
                    item_doc.append(comma)
                } else {
                    item_doc.append(comma.flat_alt(Self::nil()))
                }
            } else {
                item_doc.append(self.token_or(item.tail_comma(), Operator::Comma))
            }
        });
        let body = self.join(docs, self.line());
        let normal = self
            .indent(boundary.clone().append(body))
            .append(boundary)
            .group();
        self.minimum_inline(rigid_body, normal, MINIMUM_INLINE_WIDTH)
    }
}

impl<T> Formattable for Vec<ListItem<'_, T>>
where
    T: Formattable,
{
    fn format(&self, formatter: &Formatter) -> FormatDoc {
        if !self.is_empty() {
            let last = self.len() - 1;
            let docs = self
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let item = item.deref().format(formatter);
                    if index < last {
                        item.append(formatter.token_or(self[index].tail_comma(), Operator::Comma))
                    } else {
                        item
                    }
                })
                .collect::<Vec<_>>();
            if docs
                .iter()
                .any(|doc| formatter.render(doc.clone(), 0).contains('\n'))
            {
                return formatter.join(docs, formatter.space());
            }
        }
        formatter.list_items(self, formatter.line_(), false)
    }
}
