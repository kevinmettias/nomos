//! Proves nomos-cli's real boundary with `nomos_check_orchestration`, through the only
//! surface a `tests/` file in this bin-only crate can reach: the compiled binary's real
//! stdout, plus a direct call into the other crate's own public API.
//!
//! `src/check.rs`'s inline `run_coverage` module and `src/check/report.rs`'s inline `tests`
//! module already exercise `nomos_check_orchestration` from inside `nomos-cli`'s own `src/`
//! -- both stay in place as white-box unit tests; there is no `src/lib.rs` for a `tests/`
//! file to link against, so "the same behavior through the public surface" can only mean the
//! compiled binary's argv/stdout/exit code here, not a call into `check::Run` itself. This
//! file is that seam's public-surface counterpart.
//!
//! `check/report.rs::Print_Counts` renders `"claim: {claim}"`, where `claim` is a real
//! `nomos_check_orchestration::Claim` this crate never constructs itself -- it only ever
//! reads one back off `nomos_check_orchestration::CheckOutcome::Judged`. So a real run's
//! stdout containing exactly what that `Claim` variant's own `Display` impl produces is a
//! falsifiable proof of that render contract: if `check/report.rs` ever stopped rendering
//! this crate's own `Claim` -- printed a hand-written literal instead, say -- this assertion
//! would only keep passing by coincidence, and a rename of the variant's `Display` text would
//! catch it immediately.
//!
//! The fixture is a bare scratch tree with no `Cargo.toml` -- `check_command.rs`'s own
//! `Test_An_Admitted_Gap_Should_Be_Reported_Without_Failing` documents why that always yields
//! `Applicability::ProviderUnavailable` findings for the dependency-edges and
//! lint-diagnostics capabilities (`cargo metadata`/`cargo clippy` cannot find a manifest), and
//! `Applicability::Is_Coverage_Debt` makes that `Claim::Incomplete` deterministically, with no
//! blocking finding in play -- exit `0`, not `1`.

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

#[test]
fn Test_A_Runs_Claim_Line_Should_Match_Claims_Own_Display()
{
    let tree = Tree::New("check-orchestration-seam").With("universe.rs", "pub const TABLES: &[&str] = &[];\n");

    let ran = Run(&["check", "--root", &tree.Root()]);

    assert_eq!(ran.code, 0, "{}", ran.stdout);

    let expected_claim_line = format!("claim: {}", nomos_check_orchestration::Claim::Incomplete);
    assert!(
        ran.stdout.contains(&expected_claim_line),
        "expected the real run's stdout to contain {expected_claim_line:?}, rendered through \
         nomos_check_orchestration::Claim's own Display impl: {}",
        ran.stdout
    );
}
