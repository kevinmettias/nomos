//! The declarations themselves: every domain in the tree, against the row of the contracts
//! table it occupies.

use nomos_contracts::{DeterminismStrength, Strategy};

/// Every domain this workspace has, with the row of the contracts table it occupies.
///
/// # Why this test was renamed
///
/// It was `Test_The_Declared_Domains_Should_Be_The_Ones_This_Item_Covered`, and
/// `docs/records/OD-DETERMINISM-001` cites it by that name as the place the two undeclared
/// rows were written down. Both are declared now, so the sentence that name asserts is
/// false. A citation that resolves to a test asserting the opposite of what the citing
/// record says is worse than one that resolves to nothing, so the name moved and
/// `docs/records/OD-DETERMINISM-002` records where it went.
///
/// # What is still not covered, and why that is not a gap
///
/// One of the table's six rows has no domain in this tree: "Progress UI, logs, telemetry,
/// agent execution", the `None` row — the CLI prints, and nothing about what it prints is
/// a fact. "Correction planning and staging" was the other; `nomos-corrections` occupies it
/// now, below. That is not an omission a declaration would repair;
/// `tests/contract/tests/determinism_declarations.rs` is where the remaining row is
/// accounted for, so that a crate arriving to occupy it cannot do so silently.
#[test]
fn Test_Every_Domain_In_The_Tree_Should_Declare_And_Be_Registered()
{
    let declared = Declared_Domains();
    assert_eq!(
        declared.len(),
        13,
        "thirteen productions are covered by eleven declarations; a new producer needs a \
         row in this table and a test of its own, whether or not it also needs a \
         declaration of its own"
    );

    Each_Domain_Declares_A_Strategy_And_Is_Registered(declared);
}

/// Every domain this workspace has, with the row of the contracts table it occupies.
fn Declared_Domains() -> [(&'static str, DeterminismStrength); 13]
{
    use nomos_analysis::FactReuse;
    use nomos_corrections::CorrectionStaging;
    use nomos_lang_rust::SyntaxFactProduction;
    use nomos_lang_rust_cargo::DependencyFactProduction;
    use nomos_lang_rust_clippy::LintFactProduction;
    use nomos_lang_rust_deny::DependencyPolicyFactProduction;
    use nomos_lang_rust_scan::ScanFactProduction;
    use nomos_spec_bundle::BundleSerialization;
    use nomos_spec_project::ProjectionOutput;
    use nomos_workspace::SnapshotSerialization;

    return [
        ("syntax-fact-production", SyntaxFactProduction::STRENGTH),
        // The same declaration, discharged over the other things it covers. Entries and
        // one strategy is the shape `P10-ROLLUP-DETERMINISM` settled on: `nomos-lang-rust`
        // has fact producers occupying one row of the contracts table, so they are one
        // promise — and a promise covering more than one producer has to be run over each,
        // or the rest are covered by a sentence and measured by nothing.
        ("module-index-rollup", SyntaxFactProduction::STRENGTH),
        // The reachability offer's Materialize is the identical shape: one file's bytes in,
        // deterministic bytes out, `syn`'s own source-order traversal. `SyntaxFactProduction`
        // is reused rather than a fourth Strategy type declared for it, per its own module
        // doc's stated policy for a producer holding the same triple.
        ("controlflow-reachability-production", SyntaxFactProduction::STRENGTH),
        ("scan-fact-production", ScanFactProduction::STRENGTH),
        ("dependency-fact-production", DependencyFactProduction::STRENGTH),
        // A second, distinct provider of the same capability, over Go's own module files
        // instead of Cargo's -- its own crate declares its own Strategy type rather than
        // reusing the one above, the same way `nomos_lang_go::SyntaxFactProduction` is its
        // own type rather than a reuse of `nomos_lang_rust::SyntaxFactProduction`.
        (
            "go-dependency-fact-production",
            <nomos_lang_go_modules::DependencyFactProduction as Strategy>::STRENGTH,
        ),
        ("lint-fact-production", LintFactProduction::STRENGTH),
        (
            "dependency-policy-fact-production",
            DependencyPolicyFactProduction::STRENGTH,
        ),
        ("fact-reuse", FactReuse::STRENGTH),
        ("snapshot-serialization", SnapshotSerialization::STRENGTH),
        ("bundle-serialization", BundleSerialization::STRENGTH),
        ("projection-output", ProjectionOutput::STRENGTH),
        ("correction-staging", CorrectionStaging::STRENGTH),
    ];
}

/// Each declared domain has a test registered under its name, and measures something —
/// `DeterminismStrength::None` would be an obligation this loop discharges without ever
/// checking anything.
fn Each_Domain_Declares_A_Strategy_And_Is_Registered(declared: [(&str, DeterminismStrength); 13])
{
    use crate::harness::Test_Name_For;

    for (domain, strength) in declared
    {
        assert!(
            !Test_Name_For(domain).is_empty(),
            "{domain} declares a strategy and has no test registered"
        );
        assert_ne!(
            strength,
            DeterminismStrength::None,
            "{domain} is measured here and promises nothing, so the measurement is \
             discharging no obligation"
        );
    }
}
