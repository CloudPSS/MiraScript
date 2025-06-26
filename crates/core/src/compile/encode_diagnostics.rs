use crate::{Config, SourceDiagnostic, config::DiagnosticPositionEncoding};

use super::SerializedDiagnostics;

pub fn encode_diagnostics(
    script: &str,
    diagnostics: &[SourceDiagnostic],
    config: &Config,
) -> SerializedDiagnostics {
    let filtered = diagnostics.iter().filter(|s| {
        debug_assert!(s.range.start <= s.range.end, "Invalid diagnostic range {s}");
        debug_assert!(s.range.end <= script.len(), "Invalid diagnostic range {s}");
        if config.diagnostic_error && s.error.is_error() {
            return true;
        }
        if config.diagnostic_warning && s.error.is_warning() {
            return true;
        }
        if config.diagnostic_info && s.error.is_info() {
            return true;
        }
        if config.diagnostic_hint && s.error.is_hint() {
            return true;
        }
        if config.diagnostic_reference && s.error.is_reference() {
            return true;
        }
        if config.diagnostic_other && s.error.is_other() {
            return true;
        }
        false
    });
    if config.diagnostic_position_encoding == DiagnosticPositionEncoding::None {
        filtered
            .flat_map(|s| {
                [
                    s.range.start.try_into().unwrap(),
                    s.range.end.try_into().unwrap(),
                    s.error.code() as u32,
                ]
            })
            .collect()
    } else {
        let mut pos_to_line_col = {
            // offsets of line starts
            let mut lines = Vec::new();
            let utf16 = config.diagnostic_position_encoding == DiagnosticPositionEncoding::Utf16;
            let utf32 = config.diagnostic_position_encoding == DiagnosticPositionEncoding::Utf32;
            move |pos: usize| {
                if pos == 0 {
                    return (1u32, 1u32);
                }
                if lines.is_empty() {
                    lines.extend(
                        script
                            .lines()
                            .map(|line| line.as_ptr() as usize - script.as_ptr() as usize),
                    );
                }
                let line = lines
                    .iter()
                    .position(|&line| line > pos)
                    .unwrap_or(lines.len())
                    - 1;
                let line_start = lines[line];
                let str = &script[line_start..pos];
                let col = if utf32 {
                    str.chars().count()
                } else if utf16 {
                    str.encode_utf16().count()
                } else {
                    str.len()
                };
                (
                    (line + 1).try_into().unwrap(),
                    (col + 1).try_into().unwrap(),
                )
            }
        };
        filtered
            .flat_map(|s| {
                let start = pos_to_line_col(s.range.start);
                let end = pos_to_line_col(s.range.end);
                [start.0, start.1, end.0, end.1, s.error.code() as u32]
            })
            .collect()
    }
}
