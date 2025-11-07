use std::ops::{Deref, DerefMut};

use crate::Config;

use super::{
    DiagnosticCode, SerializedDiagnostics, SourceDiagnostic, SourceRange, encode_diagnostics,
};

pub struct DiagnosticsCollector<'s, 'c> {
    data: Vec<SourceDiagnostic>,
    pub script: &'s str,
    pub config: &'c Config,
}

impl<'s, 'c> DiagnosticsCollector<'s, 'c> {
    pub fn new(config: &'c Config, script: &'s str) -> Self {
        Self {
            config,
            script,
            data: vec![],
        }
    }

    pub fn should_push(&self, error: DiagnosticCode) -> bool {
        if error.is_error() && !self.config.diagnostic_error {
            return false;
        }
        if error.is_warning() && !self.config.diagnostic_warning {
            return false;
        }
        if error.is_info() && !self.config.diagnostic_info {
            return false;
        }
        if error.is_hint() && !self.config.diagnostic_hint {
            return false;
        }
        if error.is_reference() && !self.config.diagnostic_reference {
            return false;
        }
        if error.is_tag() && !self.config.diagnostic_tag {
            return false;
        }
        if error.is_sourcemap() && !self.config.diagnostic_sourcemap {
            return false;
        }
        true
    }

    pub fn push(&mut self, error: DiagnosticCode, range: SourceRange) {
        debug_assert!(
            range.start <= range.end,
            "Invalid diagnostic range {}",
            SourceDiagnostic::new(range, error)
        );
        debug_assert!(
            range.end <= self.script.len(),
            "Invalid diagnostic range {}",
            SourceDiagnostic::new(range, error)
        );
        if !self.should_push(error) {
            return;
        }
        self.data.push(SourceDiagnostic::new(range, error));
    }

    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = SourceDiagnostic>) {
        for diagnostic in diagnostics {
            if self.should_push(diagnostic.error) {
                self.data.push(diagnostic);
            }
        }
    }

    pub fn append(&mut self, diagnostics: &mut Vec<SourceDiagnostic>) {
        for diagnostic in diagnostics.drain(..) {
            if self.should_push(diagnostic.error) {
                self.data.push(diagnostic);
            }
        }
    }

    pub fn encode(&self) -> SerializedDiagnostics {
        encode_diagnostics(self.script, &self.data, self.config)
    }
}

impl Deref for DiagnosticsCollector<'_, '_> {
    type Target = Vec<SourceDiagnostic>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for DiagnosticsCollector<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl IntoIterator for DiagnosticsCollector<'_, '_> {
    type Item = SourceDiagnostic;
    type IntoIter = std::vec::IntoIter<SourceDiagnostic>;
    fn into_iter(self) -> std::vec::IntoIter<SourceDiagnostic> {
        self.data.into_iter()
    }
}
