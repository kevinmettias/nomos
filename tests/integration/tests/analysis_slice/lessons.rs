//! The three lessons the prototype learned expensively.
//!
//! A hole in the corpus stays visible, the fixture is the shape its own README claims, and an
//! unmeetable requirement is reported as coverage debt rather than as an absence of findings.
//! The first lesson — that a signal firing on everything is not a signal — is measurable only
//! at scale and is therefore in [`crate::scale`].

use crate::corpus::{Over_The_Precision_Corpus, Surface_Of};
use nomos_contracts::{
    Applicability, Assurance, CapabilityId, FactVariant, Guarantee, IncrementalGranularity
};

/// A hole in the corpus stays visible.
///
/// `gamma/broken.rs` cannot be read, so `gamma`'s rollup covers one of its two files. It
/// reports that rather than reporting a smaller directory, because a rollup that silently
/// omits what it could not read is indistinguishable from one whose inputs were all fine.
#[test]
fn Test_A_Rollup_Over_A_Refused_Member_Should_Report_A_Degraded_Answer()
{
    let (corpus, mut slice) = Over_The_Precision_Corpus();
    let report = slice.Run(&corpus);
    assert_eq!(
        report.refused.len(),
        1,
        "one file in the precision corpus does not parse: {:?}",
        report.refused
    );
    assert_eq!(report.degraded, vec!["gamma"]);

    let surface = Surface_Of(&slice, &corpus, "gamma");
    assert_eq!(surface.files, 1, "one of gamma's two files was readable");
    assert_eq!(
        surface.unreachable, 1,
        "and the other is counted, not dropped. A rollup reporting files=1 with \
         unreachable=0 would be claiming gamma has one file"
    );
    // The undegraded groups, as the control. If every rollup reported unreachable members
    // the assertion above would pass over a slice that could read nothing.
    for group in ["alpha", "beta"]
    {
        let whole = Surface_Of(&slice, &corpus, group);
        assert_eq!(whole.unreachable, 0, "{group} has no unreadable members");
        assert_eq!(whole.files, 2, "{group} has two files");
    }
}

/// The precision corpus's shape, asserted so its README cannot drift from it.
#[test]
fn Test_The_Precision_Corpus_Should_Have_The_Shape_Its_Readme_Claims()
{
    let (corpus, mut slice) = Over_The_Precision_Corpus();
    slice.Run(&corpus);

    for (group, files, items, public) in [("alpha", 2, 7, 3), ("beta", 2, 7, 3), ("gamma", 1, 3, 0)]
    {
        let surface = Surface_Of(&slice, &corpus, group);
        assert_eq!(
            (surface.files, surface.items, surface.public),
            (files, items, public),
            "{group} does not match the table in tests/corpus/analysis/README.md"
        );
    }
}

/// Resolution is a value, and a caller that needs more than any provider offers is told so
/// rather than served something weaker.
///
/// The composition's honesty check: the registry holds two providers, and a requirement no
/// offer reaches resolves to `MissingCapability` — coverage debt — rather than to
/// `NotApplicable`, which would be a statement about the subject that only a rule may make.
#[test]
fn Test_An_Unmeetable_Requirement_Should_Report_Coverage_Debt()
{
    use nomos_cap_syntax as syntax;
    use nomos_capability::Requirement;
    use nomos_integration_tests::Slice;
    use nomos_lang_rust as rust;

    let slice = Slice::Composed();
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Symbol,
    );
    let needs_resolution = Requirement::New(
        CapabilityId::New(syntax::CAPABILITY),
        syntax::CONTRACT_VERSION,
        guarantee,
    );

    let resolved = slice.Registry().Resolve(&needs_resolution);
    assert!(resolved.Offer().is_none());
    assert_eq!(
        resolved.Applicability(),
        Applicability::MissingCapability,
        "nothing offers this, which is coverage debt. NotApplicable would say the subject \
         does not bind the rule, and the registry is in no position to say that"
    );
    // The positive control. If resolution refused everything the assertion above would
    // pass over a composition that serves nobody.
    let servable = Requirement::New(
        CapabilityId::New(syntax::CAPABILITY),
        syntax::CONTRACT_VERSION,
        rust::Declared_Guarantee(),
    );
    assert!(slice.Registry().Resolve(&servable).Offer().is_some());
}
