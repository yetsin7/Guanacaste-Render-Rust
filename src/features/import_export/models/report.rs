//! The import report: what was read, what was skipped and why.

use std::collections::BTreeMap;

/// Severity of a report entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informative.
    Info,
    /// Something was skipped or approximated.
    Warning,
    /// Something failed.
    Error,
}

/// One kind of message with how many times it happened.
#[derive(Debug, Clone, PartialEq)]
pub struct ReportEntry {
    /// Severity.
    pub severity: Severity,
    /// Localization key of the message.
    pub key: &'static str,
    /// Arguments substituted into the message.
    pub args: Vec<String>,
    /// How many times it occurred.
    pub count: usize,
}

/// The result log of one import.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImportReport {
    /// Messages in order of first occurrence, repeated ones merged.
    pub entries: Vec<ReportEntry>,
    /// Skipped IFC entity types and their counts (properties, owner history...).
    pub skipped_types: BTreeMap<String, usize>,
    /// Products found in the file.
    pub elements_total: usize,
    /// Products that produced geometry.
    pub elements_with_geometry: usize,
    /// Products whose geometry failed or is unsupported.
    pub elements_failed: usize,
    /// Wall-clock milliseconds of the import.
    pub duration_ms: u128,
}

impl ImportReport {
    /// Records `key` with `args`; identical messages are merged and counted.
    pub fn note(&mut self, severity: Severity, key: &'static str, args: &[&str]) {
        let args: Vec<String> = args.iter().map(|text| (*text).to_owned()).collect();
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.key == key && entry.args == args)
        {
            entry.count += 1;
        } else {
            self.entries.push(ReportEntry {
                severity,
                key,
                args,
                count: 1,
            });
        }
    }

    /// Number of warnings (counting repetitions).
    pub fn warning_count(&self) -> usize {
        self.count_of(Severity::Warning)
    }

    /// Number of errors (counting repetitions).
    pub fn error_count(&self) -> usize {
        self.count_of(Severity::Error)
    }

    fn count_of(&self, severity: Severity) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.severity == severity)
            .map(|entry| entry.count)
            .sum()
    }
}
