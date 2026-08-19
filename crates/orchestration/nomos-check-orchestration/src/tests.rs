//! What this crate promises, exercised directly against [`crate::Run`] and, for the one
//! guarantee `Run`'s single entry point cannot isolate on its own, against its private
//! pieces.
//!
//! The composition here is the real one -- the real syntax provider, the real rule -- over
//! sources a test wrote by hand, which is what makes every assertion a statement about the
//! shipped seam rather than about a fixture. `nomos-cli`'s own `check` suite exercises the
//! same paths again through the compiled binary; this file is the crate's own guarantee,
//! independent of that caller ever existing.

use crate::{CheckOutcome, Claim, Run};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_contracts::{Finding, GateCategory};
use nomos_model::Subject_Of_Path;
use nomos_rules::{Check_Completeness_Mirrors, SourceFile};
use nomos_workspace::BuildVariant;

fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text);
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// This repository's own real root, for `root` -- `Run`'s dependency step runs `cargo
/// metadata` against it regardless of what `sources` a test hands in, since the dependency
/// capability is a workspace-wide fact and not a fact about any one of `sources`'s files.
/// Real on purpose, the same choice this crate's own doc comment already states for the
/// syntax half: "the real composition... which is what makes every assertion a statement
/// about the shipped seam rather than about a fixture."
fn Repository_Root() -> std::path::PathBuf
{
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .map(std::path::PathBuf::from)
        .expect("this crate sits three levels below the workspace root");
}

#[test]
fn Test_A_Clean_Tree_Should_Be_Judged_Complete_With_No_Findings()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];

    let outcome = Run(&sources, Test_Variant(), &Repository_Root());

    let CheckOutcome::Judged { findings, examined, claim } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(examined, crate::Examined { files: 1, facts: 1 });
    assert_eq!(claim, Claim::Complete);
}

/// A finding that can fail a build and a finding the run could not resolve a judgment
/// about are different axes -- `OD-COMPLETENESS-004`'s whole point, restated here because
/// this crate is now where both axes are actually decided. A phantom mirror is
/// `Applicability::Supported` (the rule read it and judged it false) and `GateCategory::
/// Blocking`, so it must move `nomos-cli`'s exit code and must NOT flip [`Claim`] --
/// [`Claim`] answers "did the run reach a verdict", not "did the verdict pass".
#[test]
fn Test_A_Blocking_Finding_Should_Still_Be_Judged_Complete()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];

    let outcome = Run(&sources, Test_Variant(), &Repository_Root());

    let CheckOutcome::Judged { findings, claim, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(!findings.is_empty(), "the phantom mirror must be reported");
    assert!(
        findings.iter().any(|finding| return finding.gate == GateCategory::Blocking),
        "{findings:?}"
    );
    assert_eq!(claim, Claim::Complete, "a blocking verdict is still a reached verdict");
}

/// A run that materializes no fact for anything it read must not be reported as judged --
/// an empty findings list here would render identically to a clean tree, which is exactly
/// the lie `nomos-cli::check`'s own vacuity guard exists to catch, one layer in.
#[test]
fn Test_A_Run_That_Materializes_No_Facts_Should_Report_NoFacts()
{
    let sources = vec![Source("broken.rs", "pub const ??? = ;")];

    let outcome = Run(&sources, Test_Variant(), &Repository_Root());

    assert!(
        matches!(outcome, CheckOutcome::NoFacts { files: 1 }),
        "a provider refusing every file must not be judged"
    );
}

/// Two sources that normalize to the same workspace member cannot both be ingested. Not a
/// case a real walk produces -- `nomos-cli::check::sources::Read_Sources` reads a real
/// directory, which cannot hand back two entries for one path -- but this crate accepts
/// `sources` from any caller, and a second adapter's own walk is not this crate's to trust.
#[test]
fn Test_Conflicting_Paths_Should_Be_Unreadable()
{
    let sources = vec![Source("a.rs", "pub fn one() {}\n"), Source("a.rs", "pub fn two() {}\n")];

    let outcome = Run(&sources, Test_Variant(), &Repository_Root());

    assert!(matches!(outcome, CheckOutcome::Unreadable), "duplicate paths must not be ingested");
}

/// The floor is the rule's, and the composition this crate assembles meets it -- the same
/// control `nomos-cli::check`'s own suite used to keep, restated here because this crate is
/// now where the composition actually lives.
#[test]
fn Test_The_Registered_Provider_Should_Satisfy_The_Rules_Floor()
{
    let sources = vec![Source("a.rs", "pub const T: &[&str] = &[];\n")];

    let outcome = Run(&sources, Test_Variant(), &Repository_Root());

    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(
        findings
            .iter()
            .all(|finding| return finding.applicability == nomos_contracts::Applicability::Supported),
        "the registered parser must serve nomos_rules::Syntax_Requirement: {findings:?}"
    );
    assert!(
        findings.iter().all(|finding| return finding.gate == GateCategory::Advisory),
        "{findings:?}"
    );
}

/// ---- the shipped composition consults a fact ----
///
/// The assertion `P10-FACT-BYPASS` turns on, and the reason this crate's own test suite
/// reaches past [`Run`] into its private [`crate::composition`] and [`crate::facts`]
/// pieces: the guarantee is that the rule's verdict depends on what the *store* holds, not
/// merely on which sources the rule was handed, and demonstrating that needs a store built
/// from one source list judged against a different, larger one. `Run` deliberately does not
/// expose that split -- a second adapter has no reason to ingest less than it judges -- so
/// this is where the split-composition guarantee is proven instead: internally, once, by
/// the crate that owns it.
#[test]
fn Test_The_Composed_Crate_Should_Resolve_A_Mirror_Through_A_Real_Fact()
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_The_Real_Provider_Found_This`.\n\
         pub const T: &[&str] = &[];\n",
    );
    let checking = Source(
        "b.rs",
        "#[cfg(test)]\nmod tests\n{\n    #[test]\n    fn Test_The_Real_Provider_Found_This()\n    {\n    }\n}\n",
    );
    let whole = vec![declaring.clone(), checking.clone()];

    let resolved = Findings_Over(&whole, &whole);
    // The store is told about the declaring file only; the rule is handed both.
    let short = Findings_Over(&[declaring], &whole);

    assert!(
        resolved.is_empty(),
        "the registered parser must find the check in b.rs: {resolved:?}"
    );
    assert!(
        short.iter().any(|finding| return finding.subject_name == "T"),
        "with b.rs's fact withheld the claim must not resolve: {short:?}"
    );
}

/// The wiring itself, proven directly: real sources flow out of the real provider over the
/// real repository root, not merely "zero findings" -- which an empty source list would
/// also produce, silently, the exact vacuity `Test_A_Clean_Tree_Should_Be_Judged_Complete_
/// With_No_Findings` cannot rule out on its own, because a `Run` that materialized nothing
/// and a `Run` that materialized a clean workspace render identically over that assertion
/// alone.
#[test]
fn Test_Materialize_Dependencies_Should_Return_Real_Workspace_Members()
{
    // What `Materialize_Dependencies` needs from a `Context` -- snapshot, variant,
    // configuration, generation -- has nothing to do with `sources`; one placeholder file
    // is enough to build a real one, the same way `Findings_Over`'s own fixtures do.
    let placeholder = [Source("placeholder.rs", "pub fn Placeholder() {}\n")];
    let registry = crate::composition::Registered().expect("fixture composition");
    let context = crate::facts::Ingested(&placeholder, &registry, Test_Variant()).expect("a single real file ingests");
    let mut store = MemoryFactStore::New();

    let (sources, findings) = crate::facts::Materialize_Dependencies(&Repository_Root(), &context, &mut store);

    assert!(findings.is_empty(), "a real workspace root must not report ProviderUnavailable: {findings:?}");
    assert!(
        sources.len() > 10,
        "this repository has far more than ten workspace members, so {} real sources is too \
         few to have exercised the provider: {sources:?}",
        sources.len()
    );
    assert!(
        sources.iter().any(|source| return source.path == "crates/rules/nomos-rules"),
        "expected nomos-rules among the real sources: {sources:?}"
    );
}

/// Ingests `ingested` into a real fact store and judges `judged` over it -- the split
/// [`crate::run::Run`] does not offer, assembled here from the crate's own private pieces.
fn Findings_Over(ingested: &[SourceFile], judged: &[SourceFile]) -> Vec<Finding>
{
    let registry = crate::composition::Registered().expect("the fixture composition is this crate's own");
    let context = crate::facts::Ingested(ingested, &registry, Test_Variant()).expect("the fixture is a valid tree");
    let mut store = MemoryFactStore::New();
    let _written = crate::facts::Materialize_Syntax(ingested, &context, &mut store);

    let mut reader = Reader::On(&store, &registry, context);
    return Check_Completeness_Mirrors(judged, &mut reader);
}
