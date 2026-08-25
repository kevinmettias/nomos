//! One test per declared domain, each discharging that domain's own declaration.
//!
//! These are the names [`Test_Name_For`] returns, so a rename here has to move with it —
//! the child process is selected by `--exact` and a name that drifted would run no test at
//! all.
//!
//! [`Test_Name_For`]: crate::harness::Test_Name_For

use crate::harness::Check;
use crate::goldens::{
    BUNDLE_GOLDEN, GO_GOLDEN, PARSED_GOLDEN, PROJECTION_GOLDEN, REACHABILITY_GOLDEN, ROLLED_GOLDEN,
    SCANNED_GOLDEN, SNAPSHOT_GOLDEN,
};
use crate::productions::{
    Correction_Production, Dependency_Production, Go_Production, Lint_Production, Parsed_Production,
    Reachability_Production, Reuse_Production, Rolled_Production, Scanned_Production, Snapshot_Production,
};
use crate::spec_productions::{Alternating, Bundle_Bytes, Projection_Bytes};
use nomos_lang_rust::SyntaxFactProduction;

#[test]
fn Test_The_Parser_Should_Meet_Its_Declared_Strategy()
{
    Check::<SyntaxFactProduction>(
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
    Check::<SyntaxFactProduction>("module-index-rollup", &Rolled_Production, ROLLED_GOLDEN);
}

/// The third producer covered by `nomos-lang-rust`'s declaration, discharged separately —
/// the identical reasoning [`Test_The_Rollup_Should_Meet_Its_Declared_Strategy`] gives for
/// the second.
#[test]
fn Test_The_Reachability_Offer_Should_Meet_Its_Declared_Strategy()
{
    Check::<SyntaxFactProduction>(
        "controlflow-reachability-production",
        &Reachability_Production,
        REACHABILITY_GOLDEN,
    );
}

#[test]
fn Test_The_Scanner_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_scan::ScanFactProduction;

    Check::<ScanFactProduction>("scan-fact-production", &Scanned_Production, SCANNED_GOLDEN);
}

#[test]
fn Test_The_Go_Provider_Should_Meet_Its_Declared_Strategy()
{
    Check::<nomos_lang_go::SyntaxFactProduction>("go-syntax-fact-production", &Go_Production, GO_GOLDEN);
}

#[test]
fn Test_The_Dependency_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_cargo::DependencyFactProduction;

    // No golden. `DependencyFactProduction` declares `CrossRun`, and `Cross_Environment_
    // Owed` therefore never reaches for one — the same shape `FactReuse` and
    // `CorrectionStaging` already take below.
    Check::<DependencyFactProduction>("dependency-fact-production", &Dependency_Production, "");
}

#[test]
fn Test_The_Lint_Provider_Should_Meet_Its_Declared_Strategy()
{
    use nomos_lang_rust_clippy::LintFactProduction;

    // No golden, the identical reason `DependencyFactProduction` has none above:
    // `LintFactProduction` declares `CrossRun`.
    Check::<LintFactProduction>("lint-fact-production", &Lint_Production, "");
}

#[test]
fn Test_The_Fact_Cache_Should_Meet_Its_Declared_Strategy()
{
    use nomos_analysis::FactReuse;

    // No golden. `FactReuse` declares `CrossRun`, and `Cross_Environment_Owed` therefore
    // never reaches for one — passing a real digest here would be a check the declaration
    // did not ask for, which is the same defect as a missing one pointed the other way.
    Check::<FactReuse>("fact-reuse", &Reuse_Production, "");
}

#[test]
fn Test_Snapshot_Serialization_Should_Meet_Its_Declared_Strategy()
{
    use nomos_workspace::SnapshotSerialization;

    Check::<SnapshotSerialization>(
        "snapshot-serialization",
        &Snapshot_Production,
        SNAPSHOT_GOLDEN,
    );
}

#[test]
fn Test_Bundle_Serialization_Should_Meet_Its_Declared_Strategy()
{
    use nomos_spec_bundle::BundleSerialization;

    Check::<BundleSerialization>(
        "bundle-serialization",
        &Alternating(Bundle_Bytes),
        BUNDLE_GOLDEN,
    );
}

#[test]
fn Test_Projection_Output_Should_Meet_Its_Declared_Strategy()
{
    use nomos_spec_project::ProjectionOutput;

    Check::<ProjectionOutput>(
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
    Check::<CorrectionStaging>("correction-staging", &Correction_Production, "");
}
