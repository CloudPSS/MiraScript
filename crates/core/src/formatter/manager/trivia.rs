use std::iter::repeat_n;

use crate::lexer::Trivia;

use super::FormatManager;

fn get_spaces_width(spaces: &str, tab_size: usize) -> usize {
    spaces
        .as_bytes()
        .iter()
        .map(|c| if *c == b'\t' { tab_size } else { 1 })
        .sum()
}

impl<'o> FormatManager<'o> {
    fn align_to_tabs<'s>(&self, str: &'s str) -> (std::iter::RepeatN<char>, &'s str) {
        let leading_spaces = str.find(|c: char| !c.is_ascii_whitespace()).unwrap_or(0);

        // `//` 后紧随的一个空格不计在内
        let leading_spaces_width =
            get_spaces_width(&str[..leading_spaces], self.tab_size).saturating_sub(1);
        let leading_spaces_tab_size =
            f64::round(leading_spaces_width as f64 / self.tab_size as f64) as usize;

        (
            self.make_indent(leading_spaces_tab_size),
            str[leading_spaces..].trim_ascii_end(),
        )
    }
    fn write_leading_block_comment(&mut self, s: &str) {
        if !self.current_line.is_empty() && !s.contains('\n') && !s.starts_with("*") {
            // 非文档的短注释，内联进当前行
            let s = s.trim();
            if s.is_empty() {
                self.current_line.push_str("/* */");
            } else {
                self.current_line.push_str("/* ");
                self.current_line.push_str(s.trim());
                self.current_line.push_str(" */");
            }
            return;
        }

        if s.is_empty() {
            // 空的 block comment
            if !self.current_leading_trivia.is_empty() {
                return;
            }
            self.current_leading_trivia.push_str("/* */");
            return;
        }

        let mut lines: Vec<_> = s.lines().collect();
        if s.ends_with('\n') {
            lines.push("");
        }
        struct CommentLine<'a> {
            data: &'a str,
            whitespaces: usize,
            has_star: bool,
        }
        let mut lines: Vec<_> = lines
            .into_iter()
            .map(|line| {
                let mut data = line.trim_ascii_start();
                let whitespaces = get_spaces_width(&line[..line.len() - data.len()], self.tab_size);
                let has_star = if data.starts_with('*') {
                    data = &data[1..];
                    true
                } else {
                    false
                };
                data = data.trim_ascii_end();
                CommentLine {
                    data,
                    whitespaces,
                    has_star,
                }
            })
            .collect();
        if self.current_leading_trivia.ends_with("*/") {
            self.current_leading_trivia.push('\n');
        }
        let doc_mode = lines[0].has_star && lines[0].whitespaces == 0;
        if let [line] = &lines[..] {
            // 单行注释
            let mut data = line.data;
            if doc_mode {
                self.current_leading_trivia.push_str("/** ");
                if data.starts_with(" ") {
                    data = &data[1..];
                }
            } else {
                self.current_leading_trivia.push_str("/* ");
                if line.has_star {
                    self.current_leading_trivia.push('*');
                }
            }
            self.current_leading_trivia.push_str(data);
            if data.is_empty() {
                self.current_leading_trivia.push_str("*/");
            } else {
                self.current_leading_trivia.push_str(" */");
            }
            return;
        }

        if !doc_mode {
            let indent = lines[1..]
                .iter()
                .map(|line| line.whitespaces)
                .min()
                .unwrap_or(0);
            for line in &mut lines[1..] {
                line.whitespaces -= indent;
            }
        }

        for (i, line) in lines.iter().enumerate() {
            let mut data = line.data;
            if doc_mode && data.starts_with(" ") {
                data = &data[1..];
            }
            if i == 0 {
                // 第一行注释
                if doc_mode {
                    self.current_leading_trivia.push_str("/** ");
                } else if data.is_empty() {
                    self.current_leading_trivia.push_str("/*");
                } else {
                    self.current_leading_trivia.push_str("/*");
                    self.current_leading_trivia
                        .extend(repeat_n(' ', usize::max(1, line.whitespaces)));
                }
                self.current_leading_trivia.push_str(data);
            } else if i != lines.len() - 1 {
                // 中间行注释
                self.current_leading_trivia.push('\n');
                if doc_mode {
                    self.current_leading_trivia.push_str(" * ");
                } else {
                    self.current_leading_trivia
                        .extend(repeat_n(' ', usize::max(1, line.whitespaces)));
                    if line.has_star {
                        self.current_leading_trivia.push('*');
                    }
                }
                self.current_leading_trivia.push_str(data);
            } else {
                // 最后一行注释
                self.current_leading_trivia.push('\n');
                if doc_mode {
                    if !data.is_empty() {
                        self.current_leading_trivia.push_str(" * ");
                    } else {
                        self.current_leading_trivia.push(' ');
                    }
                } else {
                    self.current_leading_trivia
                        .extend(repeat_n(' ', line.whitespaces));
                    if line.has_star {
                        self.current_leading_trivia.push('*');
                    }
                }
                self.current_leading_trivia.push_str(data);
                self.current_leading_trivia.push_str("*/");
            }
        }
    }
    pub fn write_leading_trivia(&mut self, trivia: &Trivia<'_>) {
        match trivia {
            Trivia::LineComment(s) => {
                if self.current_leading_trivia.ends_with("*/") {
                    self.current_leading_trivia.push('\n');
                }
                let (leading_spaces, comment_data) = self.align_to_tabs(s);
                if comment_data.is_empty() {
                    self.current_leading_trivia.push_str("//\n");
                    return;
                }
                self.current_leading_trivia.push_str("// ");
                self.current_leading_trivia.extend(leading_spaces);
                self.current_leading_trivia.push_str(comment_data);
                self.current_leading_trivia.push('\n');
            }
            Trivia::UnterminatedBlockComment(s) | Trivia::BlockComment(s) => {
                self.write_leading_block_comment(s);
            }
            Trivia::NewLine => {
                if !self.current_line.is_empty() {
                    // 只有当前行的第一个 token 可以在前方添加空行
                    return;
                }
                if self.current_leading_trivia.ends_with("\n\n")
                    || self.current_leading_trivia == "\n"
                {
                    // 不允许连续的空行
                    return;
                }
                self.current_leading_trivia.push('\n');
            }
        }
    }
    pub fn write_tailing_trivia(&mut self, trivia: &Trivia<'_>) {
        match trivia {
            Trivia::LineComment(s) => {
                if self.current_tailing_trivia.is_empty() {
                    self.current_tailing_trivia.push_str("// ");
                } else {
                    self.current_tailing_trivia.push(' ');
                }
                self.current_tailing_trivia.push_str(s.trim_ascii());
            }
            Trivia::BlockComment(s) | Trivia::UnterminatedBlockComment(s) => {
                let s = s.trim();
                if s.is_empty() {
                    return;
                }
                if self.current_tailing_trivia.is_empty() {
                    self.current_tailing_trivia.push_str("//");
                }
                for line in s.lines() {
                    self.current_tailing_trivia.push(' ');
                    self.current_tailing_trivia.push_str(line);
                }
            }
            Trivia::NewLine => (),
        }
    }
}
