use crate::lexer::Trivia;

use super::{FormatDoc, FormatManager};

impl FormatManager<'_> {
    fn block_comment(&self, contents: &str, terminated: bool) -> FormatDoc {
        let doc_mode = contents.starts_with('*');
        let contents = contents.trim();
        if !contents.contains(['\r', '\n']) {
            let contents = if doc_mode {
                contents.strip_prefix('*').unwrap_or(contents).trim()
            } else {
                contents
            };
            let mut text = if contents.is_empty() {
                "/* */".to_owned()
            } else if doc_mode {
                format!("/** {contents} */")
            } else {
                format!("/* {contents} */")
            };
            if !terminated {
                text.truncate(text.len() - 2);
            }
            return self.text(text);
        }

        let mut lines = contents.lines().map(str::trim).collect::<Vec<_>>();
        if doc_mode && lines.first().is_some_and(|line| *line == "*") {
            lines.remove(0);
        }
        let mut docs = Vec::with_capacity(lines.len() * 2 + 2);
        docs.push(self.text(if doc_mode { "/**" } else { "/*" }));
        for line in lines {
            let line = line.strip_prefix('*').unwrap_or(line).trim();
            docs.push(self.hardline());
            docs.push(self.text(if line.is_empty() {
                " *".to_owned()
            } else {
                format!(" * {line}")
            }));
        }
        if terminated {
            docs.push(self.hardline());
            docs.push(self.text(" */"));
        }
        self.concat(docs)
    }

    pub fn leading_trivia(&self, trivia: &[Trivia<'_>]) -> FormatDoc {
        let mut docs = Vec::new();
        let mut index = 0;
        while index < trivia.len() {
            match &trivia[index] {
                Trivia::LineComment(contents, _) => {
                    let contents = contents.trim_ascii();
                    docs.push(self.text(if contents.is_empty() {
                        "//".to_owned()
                    } else {
                        format!("// {contents}")
                    }));
                    docs.push(self.hardline());

                    index += 1;
                    if index < trivia.len() && matches!(trivia[index], Trivia::NewLine(_)) {
                        // 行注释的范围已经包含行尾；额外的换行表示源码中有空行。
                        docs.push(self.hardline());
                        while index < trivia.len() && matches!(trivia[index], Trivia::NewLine(_)) {
                            index += 1;
                        }
                    }
                }
                Trivia::BlockComment(contents, _) => {
                    docs.push(self.block_comment(contents, true));
                    index += 1;
                    append_block_comment_separator(self, trivia, &mut index, &mut docs);
                }
                Trivia::UnterminatedBlockComment(contents, _) => {
                    docs.push(self.block_comment(contents, false));
                    index += 1;
                    append_block_comment_separator(self, trivia, &mut index, &mut docs);
                }
                Trivia::NewLine(_) => {
                    // 一个换行已由前一个语法单元的结构分隔符表示。出现在下一个
                    // token 的 leading trivia 中的换行均是额外空行，最多保留一个。
                    docs.push(self.hardline());
                    while index < trivia.len() && matches!(trivia[index], Trivia::NewLine(_)) {
                        index += 1;
                    }
                }
            }
        }
        self.concat(docs)
    }

    pub fn tailing_trivia(&self, trivia: &[Trivia<'_>]) -> FormatDoc {
        let mut docs = Vec::new();
        for trivia in trivia {
            match trivia {
                Trivia::LineComment(contents, _) => {
                    let contents = contents.trim_ascii();
                    docs.push(self.space());
                    docs.push(self.text(if contents.is_empty() {
                        "//".to_owned()
                    } else {
                        format!("// {contents}")
                    }));
                }
                Trivia::BlockComment(contents, _) => {
                    docs.push(self.space());
                    docs.push(self.block_comment(contents, true));
                }
                Trivia::UnterminatedBlockComment(contents, _) => {
                    docs.push(self.space());
                    docs.push(self.block_comment(contents, false));
                    docs.push(self.hardline());
                }
                Trivia::NewLine(_) => {}
            }
        }
        self.concat(docs)
    }

    pub fn detached_leading_comments(&self, trivia: &[Trivia<'_>]) -> Option<FormatDoc> {
        if !trivia
            .iter()
            .any(|item| !matches!(item, Trivia::NewLine(_)))
        {
            return None;
        }

        let mut docs = Vec::new();
        let mut index = 0;
        while index < trivia.len() {
            let (comment, line_comment) = match &trivia[index] {
                Trivia::LineComment(contents, _) => {
                    let contents = contents.trim_ascii();
                    let comment = self.text(if contents.is_empty() {
                        "//".to_owned()
                    } else {
                        format!("// {contents}")
                    });
                    (comment, true)
                }
                Trivia::BlockComment(contents, _) => (self.block_comment(contents, true), false),
                Trivia::UnterminatedBlockComment(contents, _) => {
                    (self.block_comment(contents, false), false)
                }
                Trivia::NewLine(_) => {
                    index += 1;
                    continue;
                }
            };
            docs.push(comment);
            index += 1;

            let mut newline_count = 0;
            while index < trivia.len() && matches!(trivia[index], Trivia::NewLine(_)) {
                newline_count += 1;
                index += 1;
            }
            let has_next_comment = index < trivia.len();
            if has_next_comment {
                if line_comment || newline_count > 0 {
                    docs.push(self.hardline());
                } else {
                    docs.push(self.space());
                }
                if (line_comment && newline_count > 0) || (!line_comment && newline_count > 1) {
                    docs.push(self.hardline());
                }
            } else if (line_comment && newline_count > 0) || (!line_comment && newline_count > 1) {
                docs.push(self.hardline());
            }
        }
        Some(self.concat(docs))
    }
}

fn append_block_comment_separator(
    formatter: &FormatManager<'_>,
    trivia: &[Trivia<'_>],
    index: &mut usize,
    docs: &mut Vec<FormatDoc>,
) {
    if *index >= trivia.len() || !matches!(trivia[*index], Trivia::NewLine(_)) {
        docs.push(formatter.space());
        return;
    }

    let mut newline_count = 0;
    while *index < trivia.len() && matches!(trivia[*index], Trivia::NewLine(_)) {
        newline_count += 1;
        *index += 1;
    }
    docs.push(formatter.hardline());
    if newline_count > 1 {
        docs.push(formatter.hardline());
    }
}
