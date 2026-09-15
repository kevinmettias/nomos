//! Proves nomos-cli's real boundary with `nomos_contracts`, through the only surface a
//! `tests/` file in this bin-only crate can reach: the compiled binary's real stdout, plus a
//! direct construction of the other crate's own public `Finding` type.
//!
//! `src/check/report.rs`'s inline `tests` module already builds `nomos_contracts::Finding`
//! values and exercises `Coverage`/`Claim_Of` over them from inside `nomos-cli`'s own `src/`
//! -- that stays in place as a white-box unit test. This file is the public-surface
//! counterpart: `check/report.rs::Report_Findings` renders every finding with exactly
//! `writeln!(stdout, "{}", finding.Describe())`, so the text the real binary prints for a
//! known fixture must equal what `nomos_contracts::Finding::Describe()` itself produces for
//! the same rule, subject and summary -- not a hand-copied string, the actual method a real
//! `Finding` this crate never constructs is rendered through.

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

/// A phantom mirror -- a universe claiming a check that does not exist -- is the one
/// fixture shape `check_command.rs` already relies on to reach `GateCategory::Blocking`, so
/// the real completeness-mirror rule is guaranteed to emit exactly one such `Finding`, whose
/// wording `checks/mirror/verdict.rs::Unresolved_Claim` reads straight from
/// `nomos_contracts::EnforcementBreach::Phantom`'s own `Describe()`.
#[test]
fn Test_A_Real_Phantoms_Rendered_Line_Should_Equal_Findings_Own_Describe()
{
    let tree = A_Tree_Claiming_An_Absent_Check();

    let ran = Run(&["check", "--root", &tree.Root()]);

    assert_eq!(ran.code, 1, "a false claim of coverage must fail the run: {}", ran.stdout);
    assert!(ran.stdout.contains("Blocking"), "{}", ran.stdout);

    let described = The_Phantom_Claim_Describes();

    assert!(
        ran.stdout.contains(&described),
        "nomos_contracts::Finding::Describe() must reproduce a line the real binary actually \
         printed -- expected {described:?} to appear in: {}",
        ran.stdout
    );
}

/// The fixture the run above walks: a universe whose own doc comment declares it is mirrored
/// by a check that does not exist, which is the one shape whose finding the real
/// completeness-mirror rule files as blocking.
fn A_Tree_Claiming_An_Absent_Check() -> Tree
{
    return Tree::New("contracts-seam").With(
        "universe.rs",
        "/// Mirrored by `Test_Nothing_Named_This_For_Contracts_Seam`.\n\
         pub const TABLES: &[&str] = &[];\n",
    );
}

/// What `nomos_contracts::Finding::Describe()` says for the one `Finding` the real
/// completeness-mirror rule must have produced for [`A_Tree_Claiming_An_Absent_Check`].
///
/// The `Finding` is built here from the crate's own constructors -- the same rule, subject
/// and summary, with `RuleId::New("completeness-mirror")`, `subject_name` read off the
/// declared item's own name, `GateCategory::Blocking` because the claimed check resolves to
/// nothing, and the summary `nomos_contracts::EnforcementBreach::Phantom`'s own `Describe()`
/// spells verbatim -- so the text compared below is produced by that method rather than
/// copied out of the binary's output.
fn The_Phantom_Claim_Describes() -> String
{
    let finding = nomos_contracts::Finding {
        rule: nomos_contracts::RuleId::New("completeness-mirror"),
        subject: nomos_contracts::SubjectId::From_Digest(nomos_contracts::Digest128::From_Bytes(
            [0; nomos_contracts::Digest128::BYTE_LENGTH],
        )),
        subject_name: "TABLES".to_owned(),
        applicability: nomos_contracts::Applicability::Supported,
        evidence: nomos_contracts::EvidenceClass::Derived,
        gate: nomos_contracts::GateCategory::Blocking,
        summary: "`Test_Nothing_Named_This_For_Contracts_Seam` resolves to no check, so this \
                  rule is declared enforced and never runs"
            .to_owned(),
        locations: vec!["universe.rs".to_owned()],
    };

    return finding.Describe();
}
