//! This workspace's own declared architecture, copied from
//! `tests/contract/tests/boundaries/bands.rs`'s `BANDS` at the time this rule was written.
//! See [`crate::dependency`]'s own module doc for why a copy and not a shared source.
//!
//! Re-synced while building [`crate::dependency::Check_Every_Member_Declares_A_Band`]:
//! five real members had drifted out of this copy since it was written
//! (`nomos-agent-executor-ollama`, `nomos-cap-dependency-policy`, `nomos-lang-go-modules`,
//! `nomos-lang-go-package`, `nomos-lang-rust-deny`), exactly the drift `OD-RULES-003`
//! named as this copy's own cost. Shipping the coverage rule against a table already
//! known to be behind reality would have made its first real run misleading rather than
//! informative.

/// This workspace's own declared architecture, copied from
/// `tests/contract/tests/boundaries/bands.rs`'s `BANDS` at the time this rule was
/// written. See this crate's `dependency` module doc for why a copy and not a shared
/// source.
pub(super) const BANDS: &[(&str, u32)] = &[
    ("nomos-contracts", 0),
    ("nomos-model", 10),
    ("nomos-store", 12),
    ("nomos-platform", 15),
    ("nomos-platform-std", 16),
    ("nomos-workspace", 18),
    ("nomos-scope-verification", 19),
    ("nomos-ledger", 20),
    ("nomos-capability", 21),
    ("nomos-analysis", 22),
    ("nomos-cap-syntax", 23),
    ("nomos-cap-dependency", 23),
    ("nomos-cap-controlflow", 23),
    ("nomos-cap-lint", 23),
    ("nomos-cap-dependency-policy", 23),
    ("nomos-cap-naming-policy", 23),
    ("nomos-cap-limits-policy", 23),
    ("nomos-cap-scripting-policy", 23),
    ("nomos-package", 24),
    ("nomos-lang-rust", 25),
    ("nomos-lang-rust-scan", 25),
    ("nomos-lang-go", 25),
    ("nomos-lang-go-modules", 25),
    ("nomos-lang-rust-cargo", 25),
    ("nomos-lang-rust-clippy", 25),
    ("nomos-lang-rust-deny", 25),
    ("nomos-repo-standards", 25),
    ("nomos-repo-limits", 25),
    ("nomos-lang-rust-package", 26),
    ("nomos-lang-go-package", 26),
    ("nomos-model-package", 26),
    ("nomos-rule-package", 26),
    ("nomos-spec-model", 11),
    ("nomos-spec-store", 12),
    ("nomos-spec-bundle", 13),
    ("nomos-spec-ingest", 13),
    ("nomos-spec-validate", 14),
    ("nomos-spec-project", 14),
    ("nomos-rules", 30),
    ("nomos-corrections", 35),
    ("nomos-agent-contracts", 36),
    ("nomos-agent-executor-claude-code", 37),
    ("nomos-model-backend-ollama", 37),
    ("nomos-work-orchestration", 40),
    ("nomos-check-orchestration", 40),
    ("nomos-spec-orchestration", 40),
    ("nomos-gate-orchestration", 41),
    ("nomos-cli", 90),
    ("nomos-api", 90),
    ("nomos-surface-provenance", 91),
    ("nomos-contract-tests", 100),
    ("nomos-integration-tests", 100),
];

#[must_use]
pub(super) fn Declared_Band(name: &str) -> Option<u32>
{
    return BANDS
        .iter()
        .find(|(crate_name, _)| *crate_name == name)
        .map(|(_, band)| *band);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Every_Entry_Should_Be_Findable_By_Name()
    {
        for (name, band) in BANDS
        {
            assert_eq!(Declared_Band(name), Some(*band), "{name}");
        }
    }

    #[test]
    fn Test_An_Unknown_Name_Should_Have_No_Declared_Band()
    {
        assert_eq!(Declared_Band("nomos-does-not-exist"), None);
    }
}
