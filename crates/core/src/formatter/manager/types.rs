use std::fmt;

use pretty::{BoxDoc, Render, RenderAnnotated};

use crate::formatter::FormatOptions;

#[derive(Debug, Clone)]
pub(crate) enum FormatAnnotation {
    SourceText,
}

pub(crate) type FormatDoc = BoxDoc<'static, FormatAnnotation>;

pub(crate) trait Formattable {
    fn format(&self, formatter: &FormatManager) -> FormatDoc;
}

impl<T: Formattable> Formattable for Box<T> {
    fn format(&self, formatter: &FormatManager) -> FormatDoc {
        self.as_ref().format(formatter)
    }
}

impl<T: Formattable> Formattable for Option<T> {
    fn format(&self, formatter: &FormatManager) -> FormatDoc {
        self.as_ref()
            .map_or_else(FormatManager::nil, |value| value.format(formatter))
    }
}

#[derive(Debug)]
pub(crate) struct FormatManager<'o> {
    pub(super) options: &'o FormatOptions,
}

impl<'o> FormatManager<'o> {
    pub fn new(options: &'o FormatOptions) -> Self {
        Self { options }
    }

    pub fn nil() -> FormatDoc {
        BoxDoc::nil()
    }

    pub fn text(&self, text: impl Into<String>) -> FormatDoc {
        BoxDoc::text(text.into())
    }

    pub fn source_text(&self, text: &str) -> FormatDoc {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        BoxDoc::nesting(move |nesting| source_text_doc(&normalized, nesting))
    }

    pub fn space(&self) -> FormatDoc {
        BoxDoc::space()
    }

    pub fn line(&self) -> FormatDoc {
        BoxDoc::line()
    }

    pub fn line_(&self) -> FormatDoc {
        BoxDoc::line_()
    }

    pub fn hardline(&self) -> FormatDoc {
        BoxDoc::hardline()
    }

    pub fn minimum_inline(
        &self,
        inline: FormatDoc,
        normal: FormatDoc,
        minimum_width: usize,
    ) -> FormatDoc {
        let inline_text = self.render(inline.clone(), 0);
        if inline_text.contains('\n') || inline_text.chars().count() > minimum_width {
            return normal;
        }
        let line_width = self.options.line_width;
        FormatDoc::column(move |column| {
            let available = line_width.saturating_sub(column);
            if line_width >= minimum_width * 2 && available < minimum_width {
                inline.clone()
            } else {
                normal.clone()
            }
        })
    }

    pub fn indent(&self, doc: FormatDoc) -> FormatDoc {
        doc.nest(self.options.tab_size as isize)
    }

    pub fn concat<I>(&self, docs: I) -> FormatDoc
    where
        I: IntoIterator<Item = FormatDoc>,
    {
        docs.into_iter().fold(Self::nil(), FormatDoc::append)
    }

    pub fn join<I>(&self, docs: I, separator: FormatDoc) -> FormatDoc
    where
        I: IntoIterator<Item = FormatDoc>,
    {
        BoxDoc::intersperse(docs, separator)
    }

    pub fn render(&self, doc: FormatDoc, initial_column: usize) -> String {
        let prefix = " ".repeat(initial_column);
        let doc = if initial_column == 0 {
            doc
        } else {
            self.text(prefix.clone())
                .append(doc.nest(initial_column as isize))
        };
        let mut rendered = AnnotatedString::default();
        doc.render_raw(self.options.line_width, &mut rendered)
            .expect("rendering into String cannot fail");
        let mut rendered = rendered.finish();
        if initial_column != 0 {
            debug_assert!(rendered.starts_with(&prefix));
            rendered.drain(..prefix.len());
        }
        if self.options.use_spaces {
            rendered
        } else {
            convert_leading_spaces_to_tabs(&rendered, self.options.tab_size)
        }
    }
}

fn source_text_doc(text: &str, nesting: usize) -> FormatDoc {
    let mut segments = text.split_inclusive('\n');
    let Some(first) = segments.next() else {
        return BoxDoc::nil();
    };
    let (first, first_has_newline) = first
        .strip_suffix('\n')
        .map_or((first, false), |line| (line, true));
    let first = BoxDoc::text(first.to_owned()).annotate(FormatAnnotation::SourceText);
    if !first_has_newline {
        return first;
    }

    let rest = segments.fold(BoxDoc::hardline(), |doc, segment| {
        let (line, has_newline) = segment
            .strip_suffix('\n')
            .map_or((segment, false), |line| (line, true));
        let doc = doc.append(BoxDoc::text(line.to_owned()).annotate(FormatAnnotation::SourceText));
        if has_newline {
            doc.append(BoxDoc::hardline())
        } else {
            doc
        }
    });
    first.append(rest.nest(-(nesting as isize)))
}

#[derive(Default)]
struct AnnotatedString {
    text: String,
    protected: Vec<bool>,
    source_depth: usize,
}

impl AnnotatedString {
    fn finish(self) -> String {
        let mut output = String::with_capacity(self.text.len());
        let mut start = 0;
        for (index, byte) in self.text.bytes().enumerate() {
            if byte != b'\n' {
                continue;
            }
            let mut end = index;
            while end > start
                && matches!(self.text.as_bytes()[end - 1], b' ' | b'\t' | b'\r')
                && !self.protected[end - 1]
            {
                end -= 1;
            }
            output.push_str(&self.text[start..end]);
            output.push('\n');
            start = index + 1;
        }
        let mut end = self.text.len();
        while end > start
            && matches!(self.text.as_bytes()[end - 1], b' ' | b'\t' | b'\r')
            && !self.protected[end - 1]
        {
            end -= 1;
        }
        output.push_str(&self.text[start..end]);
        output
    }
}

impl Render for AnnotatedString {
    type Error = fmt::Error;

    fn write_str(&mut self, text: &str) -> Result<usize, Self::Error> {
        self.text.push_str(text);
        self.protected
            .extend(std::iter::repeat_n(self.source_depth > 0, text.len()));
        Ok(text.len())
    }

    fn fail_doc(&self) -> Self::Error {
        fmt::Error
    }
}

impl RenderAnnotated<'_, FormatAnnotation> for AnnotatedString {
    fn push_annotation(&mut self, annotation: &FormatAnnotation) -> Result<(), Self::Error> {
        match annotation {
            FormatAnnotation::SourceText => self.source_depth += 1,
        }
        Ok(())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        debug_assert!(self.source_depth > 0);
        self.source_depth -= 1;
        Ok(())
    }
}

fn convert_leading_spaces_to_tabs(input: &str, tab_size: usize) -> String {
    let mut output = String::with_capacity(input.len());
    for segment in input.split_inclusive('\n') {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        let spaces = line.bytes().take_while(|byte| *byte == b' ').count();
        output.extend(std::iter::repeat_n('\t', spaces / tab_size));
        output.extend(std::iter::repeat_n(' ', spaces % tab_size));
        output.push_str(&line[spaces..]);
        if segment.ends_with('\n') {
            output.push('\n');
        }
    }
    output
}
