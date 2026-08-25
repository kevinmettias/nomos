//! One workspace member's full set of lint diagnostics.

use super::lint_diagnostic::LintDiagnostic;

/// One workspace member's full set of lint diagnostics — empty when the tool found none,
/// the same "clean is a real answer, not an absence" shape [`nomos_cap_dependency::
/// DependencyPayload`] already has for a member with no edges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsPayload
{
    pub package: String,
    pub diagnostics: Vec<LintDiagnostic>,
}
