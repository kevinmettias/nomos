//! One test per declared domain, each discharging that domain's own declaration.
//!
//! These are the names [`Test_Name_For`] returns, so a rename here has to move with it —
//! the child process is selected by `--exact` and a name that drifted would run no test at
//! all.
//!
//! [`Test_Name_For`]: crate::harness::Test_Name_For

use crate::harness::Assert_Meets_Declared_Strategy;
use crate::goldens::{
    BUNDLE_GOLDEN, GO_GOLDEN, PARSED_GOLDEN, PROJECTION_GOLDEN, REACHABILITY_GOLDEN, ROLLED_GOLDEN,
    SCANNED_GOLDEN, SNAPSHOT_GOLDEN,
};
use crate::productions::{
    Coderabbit_Review_Finding_Production, Copy_Clones_Production, Correction_Production, Dependency_Policy_Production,
    Dependency_Production, Go_Dependency_Production, Go_Production, Limits_Policy_Production, Lint_Production,
    Naming_Policy_Production, Nested_Locks_Production, Parsed_Production, Reachability_Production,
    Requirement_Trace_Production, Reuse_Production, Rolled_Production, Scanned_Production, Scripting_Policy_Production,
    Goals_Policy_Production, Snapshot_Production, Words_Policy_Production,
};
use crate::spec_productions::{Alternating, Bundle_Bytes, Projection_Bytes};
use nomos_lang_rust::SyntaxFactProduction;

#[test]
fn Test_The_Parser_Should_Meet_Its_Declared_Strategy()
{
    Assert_Meets_Declared_Strategy::<SyntaxFactProduction>(
        "syntax-fact-production",
        &Parsed_Production,
        PARSED_GOLDEN,
    );
}

/// The second producer covered by `nomos-lang-rust`'s declaration, discharged separately.
///
/// Same `Strategy`, because it is the same execution domain holding the same triple —
/// `SyntaxFactProduction`'s doc says why one declaration covers both. What must not be
/// shared is the *production*: a declaration is only as good as what the harness runs it
/// over, and this is the run that makes the crate's promise true of the rollup rather than
/// merely stated about it.
#[test]
fn Test_The_Rollup_Should_Meet_Its_Declared_Strategy()
{
    Assert_Meets_Declared_Strategy::<SyntaxFactProduction>("module-index-rollup", &Rolled_Production, ROLLED_GOLDEN);
}

/// The third producer covered by `nomos-lang-rust`'s declaration, discharged separately —
/// the identical reasoning [`Test_The_Rollup_Should_Meet_Its_Declared_Strategy`] gives for
/// the second.
#[test]
fn Test_The_Reachability_Offer_Should_Meet_Its_Declared_Strategy()
{
    Assert_Meets_Declared_Strategy::<SyntaxFactProduction>(
        "controlflow-reachability-production",
        &Reachability_Production,
        REACHABILITY_GOLDEN,
    );
}

#[test]
fn Test_The_Scanner_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_scan::ScanFactProduction;

    Assert_Meets_Declared_Strategy::<ScanFactProduction>("scan-fact-production", &Scanned_Production, SCANNED_GOLDEN);
}

#[test]
fn Test_The_Go_Provider_Should_Meet_Its_Declared_Strategy()
{
    Assert_Meets_Declared_Strategy::<nomos_lang_go::SyntaxFactProduction>(
        "go-syntax-fact-production",
        &Go_Production,
        GO_GOLDEN,
    );
}

#[test]
fn Test_The_Dependency_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_cargo::DependencyFactProduction;

    // No golden. `DependencyFactProduction` declares `CrossRun`, and `Cross_Environment_
    // Owed` therefore never reaches for one — the same shape `FactReuse` and
    // `CorrectionStaging` already take below.
    Assert_Meets_Declared_Strategy::<DependencyFactProduction>(
        "dependency-fact-production",
        &Dependency_Production,
        "",
    );
}

#[test]
fn Test_The_Go_Dependency_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_go_modules::DependencyFactProduction;

    // No golden, the identical reason `nomos_lang_rust_cargo::DependencyFactProduction`
    // has none above: this crate's own `DependencyFactProduction` declares `CrossRun` too.
    Assert_Meets_Declared_Strategy::<DependencyFactProduction>(
        "go-dependency-fact-production",
        &Go_Dependency_Production,
        "",
    );
}

#[test]
fn Test_The_Lint_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_clippy::LintFactProduction;

    // No golden, the identical reason `DependencyFactProduction` has none above:
    // `LintFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<LintFactProduction>("lint-fact-production", &Lint_Production, "");
}

#[test]
fn Test_The_Dependency_Policy_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_deny::DependencyPolicyFactProduction;

    // No golden, the identical reason `LintFactProduction` has none above:
    // `DependencyPolicyFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<DependencyPolicyFactProduction>(
        "dependency-policy-fact-production",
        &Dependency_Policy_Production,
        "",
    );
}

#[test]
fn Test_The_Copy_Clones_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_compiler::CloneOnCopyFactProduction;

    // No golden, the identical reason `DependencyPolicyFactProduction` has none above:
    // `CloneOnCopyFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<CloneOnCopyFactProduction>("copy-clones-fact-production", &Copy_Clones_Production, "");
}

#[test]
fn Test_The_Nested_Locks_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_compiler::NestedLockFactProduction;

    // No golden, the identical reason `CloneOnCopyFactProduction` has none above:
    // `NestedLockFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<NestedLockFactProduction>("nested-locks-fact-production", &Nested_Locks_Production, "");
}

#[test]
fn Test_The_Limits_Policy_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_repo_policy::limits::LimitsPolicyFactProduction;

    // No golden, the identical reason `DependencyPolicyFactProduction` has none above:
    // `LimitsPolicyFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<LimitsPolicyFactProduction>(
        "limits-policy-fact-production",
        &Limits_Policy_Production,
        "",
    );
}

#[test]
fn Test_The_Naming_Policy_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_repo_policy::naming::NamingPolicyFactProduction;

    // No golden, the identical reason `LimitsPolicyFactProduction` has none above:
    // `NamingPolicyFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<NamingPolicyFactProduction>(
        "naming-policy-fact-production",
        &Naming_Policy_Production,
        "",
    );
}

#[test]
fn Test_The_Scripting_Policy_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_repo_policy::scripting::ScriptingPolicyFactProduction;

    // No golden, the identical reason `NamingPolicyFactProduction` has none above:
    // `ScriptingPolicyFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<ScriptingPolicyFactProduction>(
        "scripting-policy-fact-production",
        &Scripting_Policy_Production,
        "",
    );
}

#[test]
fn Test_The_Words_Policy_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_repo_policy::words::WordsPolicyFactProduction;

    // No golden, the identical reason `ScriptingPolicyFactProduction` has none above:
    // `WordsPolicyFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<WordsPolicyFactProduction>(
        "words-policy-fact-production",
        &Words_Policy_Production,
        "",
    );
}

#[test]
fn Test_The_Goals_Policy_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_repo_policy::goals::GoalsPolicyFactProduction;

    // No golden, the identical reason `WordsPolicyFactProduction` has none above:
    // `GoalsPolicyFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<GoalsPolicyFactProduction>(
        "goals-policy-fact-production",
        &Goals_Policy_Production,
        "",
    );
}

#[test]
fn Test_The_Requirement_Trace_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_cap_requirement_trace::RequirementTraceFactProduction;

    // No golden, the identical reason `GoalsPolicyFactProduction` has none above:
    // `RequirementTraceFactProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<RequirementTraceFactProduction>(
        "requirement-trace-fact-production",
        &Requirement_Trace_Production,
        "",
    );
}

#[test]
fn Test_The_Coderabbit_Connector_Should_Meet_Its_Declared_Strategy()
{
    use nomos_connector_coderabbit::ReviewFindingProduction;

    // No golden, the identical reason `GoalsPolicyFactProduction` has none above:
    // `ReviewFindingProduction` declares `CrossRun`.
    Assert_Meets_Declared_Strategy::<ReviewFindingProduction>(
        "connector-review-finding-fact-production",
        &Coderabbit_Review_Finding_Production,
        "",
    );
}

#[test]
fn Test_The_Fact_Cache_Should_Meet_Its_Declared_Strategy()
{
    use nomos_analysis::FactReuse;

    // No golden. `FactReuse` declares `CrossRun`, and `Cross_Environment_Owed` therefore
    // never reaches for one — passing a real digest here would be a check the declaration
    // did not ask for, which is the same defect as a missing one pointed the other way.
    Assert_Meets_Declared_Strategy::<FactReuse>("fact-reuse", &Reuse_Production, "");
}

#[test]
fn Test_Snapshot_Serialization_Should_Meet_Its_Declared_Strategy()
{
    use nomos_workspace::SnapshotSerialization;

    Assert_Meets_Declared_Strategy::<SnapshotSerialization>(
        "snapshot-serialization",
        &Snapshot_Production,
        SNAPSHOT_GOLDEN,
    );
}

#[test]
fn Test_Bundle_Serialization_Should_Meet_Its_Declared_Strategy()
{
    use nomos_spec_bundle::BundleSerialization;

    Assert_Meets_Declared_Strategy::<BundleSerialization>(
        "bundle-serialization",
        &Alternating(Bundle_Bytes),
        BUNDLE_GOLDEN,
    );
}

#[test]
fn Test_Projection_Output_Should_Meet_Its_Declared_Strategy()
{
    use nomos_spec_project::ProjectionOutput;

    Assert_Meets_Declared_Strategy::<ProjectionOutput>(
        "projection-output",
        &Alternating(Projection_Bytes),
        PROJECTION_GOLDEN,
    );
}

#[test]
fn Test_Corrections_Should_Meet_Their_Declared_Strategy()
{
    use nomos_corrections::CorrectionStaging;

    // No golden. `CorrectionStaging` declares `CrossRun`, and `Cross_Environment_Owed`
    // therefore never reaches for one — passing a real digest here would be a check the
    // declaration did not ask for, which is the same defect as a missing one pointed the
    // other way.
    Assert_Meets_Declared_Strategy::<CorrectionStaging>("correction-staging", &Correction_Production, "");
}
