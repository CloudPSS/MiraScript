use crate::{Config, SourceDiagnostic, config::DiagnosticPositionEncoding};

pub type SerializedDiagnostics = Vec<u32>;

fn encode_diagnostics_none(diagnostics: &[SourceDiagnostic]) -> SerializedDiagnostics {
    diagnostics
        .iter()
        .flat_map(|s| {
            [
                s.range.start.try_into().unwrap(),
                s.range.end.try_into().unwrap(),
                s.error.code() as u32,
            ]
        })
        .collect()
}

fn pos_to_line_col(script: &str, config: &Config) -> impl Fn(usize) -> (u32, u32) {
    let code_point_counter =
        if config.diagnostic_position_encoding == DiagnosticPositionEncoding::Utf16 {
            |s: &str| s.encode_utf16().count()
        } else if config.diagnostic_position_encoding == DiagnosticPositionEncoding::Utf32 {
            |s: &str| s.chars().count()
        } else {
            |s: &str| s.len()
        };

    const LONG_LINE_THRESHOLD: usize = 1024;
    struct Offset {
        pub offset: usize,
        pub line: usize,
        pub col: usize,
    }
    // offsets of line starts
    let offsets: Vec<_> = script
        .lines()
        .enumerate()
        .flat_map(|(n, line)| {
            let line_no = n + 1;
            let offset = line.as_ptr() as usize - script.as_ptr() as usize;
            let mut pos = Offset {
                offset,
                line: line_no,
                col: 1,
            };
            if line.len() <= LONG_LINE_THRESHOLD {
                return vec![pos];
            }
            let mut markers = vec![];
            while pos.offset - offset < line.len() {
                let next_line_pos =
                    line.floor_char_boundary(pos.offset - offset + LONG_LINE_THRESHOLD);
                let next_offset = offset + next_line_pos;
                let next_col =
                    pos.col + code_point_counter(&line[pos.offset - offset..next_line_pos]);
                markers.push(std::mem::replace(
                    &mut pos,
                    Offset {
                        offset: next_offset,
                        line: line_no,
                        col: next_col,
                    },
                ));
            }
            markers
        })
        .collect();
    move |pos: usize| {
        if pos == 0 {
            return (1u32, 1u32);
        }
        let offset = offsets
            .iter()
            .position(|line| line.offset > pos)
            .unwrap_or(offsets.len())
            - 1;
        let offset = &offsets[offset];
        let str = &script[offset.offset..pos];
        let code_point_count = code_point_counter(str);
        (
            offset.line.try_into().unwrap(),
            (offset.col + code_point_count).try_into().unwrap(),
        )
    }
}

pub fn encode_diagnostics(
    script: &str,
    diagnostics: &[SourceDiagnostic],
    config: &Config,
) -> SerializedDiagnostics {
    if diagnostics.is_empty() {
        return vec![];
    }

    if config.diagnostic_position_encoding == DiagnosticPositionEncoding::None {
        return encode_diagnostics_none(diagnostics);
    }

    let pos_to_line_col = pos_to_line_col(script, config);
    diagnostics
        .iter()
        .flat_map(|s| {
            let start = pos_to_line_col(s.range.start);
            let end = pos_to_line_col(s.range.end);
            [start.0, start.1, end.0, end.1, s.error.code() as u32]
        })
        .collect()
}
