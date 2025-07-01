use crate::lexer::Trivia;

use super::FormatManager;

impl<'o> FormatManager<'o> {
    fn align_to_tabs<'s>(&self, str: &'s str) -> (std::iter::RepeatN<char>, &'s str) {
        let leading_spaces = str.find(|c: char| !c.is_ascii_whitespace()).unwrap_or(0);

        // `//` 后紧随的一个空格不计在内
        let leading_spaces_width = str[..leading_spaces]
            .chars()
            .map(|c| if c == '\t' { self.tab_size } else { 1 })
            .sum::<usize>()
            .saturating_sub(1);
        let leading_spaces_tab_size =
            f64::round(leading_spaces_width as f64 / self.tab_size as f64) as usize;

        (
            self.make_indent(leading_spaces_tab_size),
            str[leading_spaces..].trim_ascii_end(),
        )
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
                if self.current_leading_trivia.is_empty() && !s.contains('\n') {
                    // 短注释，内联进当前行
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
                for (i, line) in s.lines().enumerate() {
                    if i == 0 {
                        if self.current_leading_trivia.ends_with("*/") {
                            self.current_leading_trivia.push('\n');
                        }
                        if line.starts_with("*") {
                            self.current_leading_trivia.push_str("/*");
                        } else {
                            self.current_leading_trivia.push_str("/* ");
                        }
                    } else {
                        self.current_leading_trivia.push('\n');
                    }
                    let mut line = line.trim();
                    if !line.is_empty() {
                        if i != 0 {
                            if line.starts_with("* ") {
                                line = &line[2..];
                            } else if line.starts_with("*") {
                                line = &line[1..];
                            }
                            self.current_leading_trivia.push_str(" * ");
                        }
                        self.current_leading_trivia.push_str(line);
                    }
                }
                if s.ends_with('\n') {
                    self.current_leading_trivia.push('\n');
                }
                self.current_leading_trivia.push_str(" */");
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
