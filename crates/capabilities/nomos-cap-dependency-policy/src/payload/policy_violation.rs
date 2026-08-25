//! One violation, as the tool itself reported it.

use super::policy_severity::PolicySeverity;

/// One violation, as the tool itself reported it.
///
/// No crate or version attribution: unlike a lint diagnostic's primary span, a `cargo
/// deny` violation does not always name one crate — a `duplicate` finding names several
/// (the whole version-conflict subgraph, `cargo deny --format json`'s own `graphs` field),
/// a `license-not-encountered` finding names a license expression rather than a crate at
/// all (`labels[0].span`), and nothing in the stream promises either field a stable shape
/// across every `bans`/`licenses`/`sources` code this tool can report. `code` and
/// `message` are the two fields every real diagnostic this reader has observed carries
/// unconditionally, verified directly against this workspace's own output before this
/// type was written — inventing a crate-attribution field neither field's presence
/// actually supports would be the same overclaim `nomos_cap_lint::LintDiagnostic`'s own
/// module doc already declines for a column-precise span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyViolation
{
    pub severity: PolicySeverity,
    /// The tool's own identifier for this violation's own kind — `duplicate`,
    /// `license-not-encountered`, `banned`, for instance. Always present: every real
    /// diagnostic `cargo deny check bans licenses sources` reports names one.
    pub code: String,
    /// The tool's own summary, single-line: an embedded newline is collapsed to a space
    /// by whichever provider encodes this, the same normalization
    /// `nomos_cap_lint::LintDiagnostic::message` applies for the identical reason — one
    /// line, one record.
    pub message: String,
}
