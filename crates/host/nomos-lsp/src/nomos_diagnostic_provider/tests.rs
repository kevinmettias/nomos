//! What this crate still decides, now that the protocol is the engine's.
//!
//! # Why the loop is no longer asserted here
//!
//! It is not this crate's any more. That a judged file is published, that a file which went
//! clean is published empty so its markers clear, that a file nothing ever published is not
//! cleared for no reason, that a one-based line becomes a zero-based span -- all of it moved
//! down with the server and is asserted in `xvpe-language-server-backend-lsp`'s own suite.
//!
//! What remains is the half that is genuinely this workspace's, and it is the half that was
//! never about the protocol in the first place: that the workspace and the fact store are
//! *reused* across two judgements rather than rebuilt, which is what `OD-ANALYSIS-009`'s
//! second amendment named this crate as the case for.

use super::*;

/// A fresh, empty directory under the system temporary directory.
fn Root(name: &str) -> std::path::PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("creates a fresh directory");

    return root;
}

#[test]
fn Test_Judging_Twice_Should_Reuse_The_Same_Workspace_And_Advance_Its_Generation_On_A_Real_Edit()
{
    let root = Root("nomos-lsp-provider-reuse-edited-root");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("the fresh root above was just created");

    let mut provider = NomosDiagnosticProvider::New();

    let _ignored = provider.Diagnose(&root);
    let generation_after_first =
        provider.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\npub fn Also_Ok() {}\n").expect("the fresh root above was just created");
    let _ignored = provider.Diagnose(&root);
    let generation_after_second =
        provider.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

    let _ignored = std::fs::remove_dir_all(&root);
    assert!(
        generation_after_second > generation_after_first,
        "a real edit reusing the same workspace must advance its generation, not repeat it: \
         {generation_after_first:?} then {generation_after_second:?}"
    );
}

#[test]
fn Test_Judging_An_Untouched_Tree_Twice_Should_Not_Advance_The_Generation_A_Second_Time()
{
    let root = Root("nomos-lsp-provider-reuse-untouched-root");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("the fresh root above was just created");

    let mut provider = NomosDiagnosticProvider::New();

    let _ignored = provider.Diagnose(&root);
    let generation_after_first =
        provider.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

    let _ignored = provider.Diagnose(&root);
    let generation_after_second =
        provider.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(
        generation_after_second, generation_after_first,
        "an untouched file must not advance the generation a second time: \
         {generation_after_first:?} then {generation_after_second:?}"
    );
}

#[test]
fn Test_A_Root_That_Cannot_Be_Walked_Should_Report_Nothing()
{
    let mut provider = NomosDiagnosticProvider::New();

    let judged = provider.Diagnose(std::path::Path::new("a-directory-that-does-not-exist"));

    assert!(judged.is_empty(), "a root that is not a directory yields no judgement");
}

/// Two sources, four judgements: cold, unchanged, one source moved, then both moved.
///
/// # Why four and not two
///
/// A second unchanged judgement answering what the first answered is consistent with a
/// provider that reused everything and with one that recomputed everything, which is the
/// same vacuity `tests/incremental_equivalence.rs` states about its own comparisons. The
/// third judgement is what separates them, because only a provider that really kept the
/// store can charge for one source and not the other. The fourth is what lets the third be
/// read without pinning a number: it moves both sources, so the fixture measures what one
/// source costs instead of this test declaring it.
///
/// # Why the assertions are relations and never a fact count
///
/// [`nomos_analysis::MemoryFactStore::Materializations`] counts every store write, and a
/// judgement of this fixture writes three different kinds: one fact per source per per-source
/// family, and one per whole-workspace family. How many of each there are is a property of
/// what is composed, not of reuse, so every assertion below is a relation between the deltas
/// and none of them names a total.
///
/// This test used to subtract instead, and that arithmetic is what made it red. It read the
/// second delta as a *per-call cost* -- the doc it carried said "the dependency, lint and
/// policy providers run their own subprocess against `root` and write again on every call, so
/// an unchanged judgement here costs something rather than nothing" -- and recovered the
/// per-source cost as `cold - unchanged`. That was true until `6bb62542`
/// (`P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY`) gave every family the currency check only
/// the syntax family had, after which an unchanged judgement writes nothing at all and the
/// subtraction returns the whole cold total. Measured with this test's own body byte-identical
/// at both revisions: at `6bb62542^` cold 12, unchanged 10, edited 11, so `cold - unchanged`
/// was 2 and the test passed; at `6bb62542` cold 12, unchanged 0, edited 2, so it read 12 and
/// failed. The reuse was not broken by that commit -- it was made total, and the test's model
/// of it went stale.
///
/// The second thing the subtraction hid is why the number it produced must not simply be
/// raised. Before `6bb62542` the reachability family was rewritten on every call, so it sat
/// inside the per-call term and one moved source cost exactly one write. It now proves
/// currency like the rest, so one moved source costs one write *per per-source family* --
/// two today, `nomos.cap.syntax.items` and `nomos.cap.controlflow.reachability`. Writing `2`
/// here would pin a family count this test has no business asserting and would go red the day
/// a third per-source family is composed, which is the failure the old doc already refused.
#[test]
fn Test_A_Second_Judgement_Should_Re_Derive_A_Fact_Only_For_The_Source_That_Moved()
{
    let root = Root("nomos-lsp-provider-selective-invalidation");
    let mut provider = NomosDiagnosticProvider::New();

    let counted = Counted_By_Judging_Four_Times(&root, &mut provider);

    let _ignored = std::fs::remove_dir_all(&root);
    Assert_Only_The_Moved_Source_Was_Charged(&counted);
}

/// How many sources the fixture holds, which is also the ratio the last judgement must cost
/// over the one before it. Named rather than written as `2` at the assertion, because the
/// number is the fixture's own shape rather than a value that means only itself.
const SOURCES_IN_THE_FIXTURE: u32 = 2;

/// What judging one fixture four times cost, in
/// [`nomos_analysis::MemoryFactStore::Materializations`] writes: the cold judgement's own
/// total, and then each later judgement's own extra over the one before it.
///
/// Each field after the first is a delta against the running total at the moment the
/// judgement before it finished, so they are independent of one another and none of them
/// contains another's cost.
struct MaterializationCounts
{
    cold: u32,
    unchanged: u32,
    one_source_moved: u32,
    both_sources_moved: u32,
}

/// Writes the two-source fixture under `root` and judges it four times through `provider` --
/// cold, unchanged, once after `a.rs` gained an item, and once more after both sources did --
/// returning the four counts the assertions above compare.
///
/// Every edit writes content the fixture has not held before. A source rewritten with the
/// bytes it already carries has not moved, and a judgement of it would be a third unchanged
/// call wearing an edit's name.
///
/// The cold judgement's own emptiness is asserted here rather than handed back: a count read
/// off a tree nothing ever judged is a number about nothing, and this is the only place that
/// can say so while the judgement itself is still in hand.
fn Counted_By_Judging_Four_Times(root: &std::path::Path, provider: &mut NomosDiagnosticProvider) -> MaterializationCounts
{
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("the fresh root above was just created");
    std::fs::write(root.join("b.rs"), "pub fn Fine() {}\n").expect("the fresh root above was just created");

    let first = provider.Diagnose(root);
    assert!(!first.is_empty(), "the cold judgement found nothing, so every count below is about a tree that was never really judged");
    let cold = provider.store.Materializations();

    let _ignored = provider.Diagnose(root);
    let after_unchanged = provider.store.Materializations();
    let unchanged = after_unchanged.saturating_sub(cold);

    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\npub fn Added() {}\n").expect("the fresh root above was just created");
    let _ignored = provider.Diagnose(root);
    let after_one = provider.store.Materializations();
    let one_source_moved = after_one.saturating_sub(after_unchanged);

    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\npub fn Added() {}\npub fn Third() {}\n").expect("the fresh root above was just created");
    std::fs::write(root.join("b.rs"), "pub fn Fine() {}\npub fn Also_Fine() {}\n").expect("the fresh root above was just created");
    let _ignored = provider.Diagnose(root);
    let both_sources_moved = provider.store.Materializations().saturating_sub(after_one);

    return MaterializationCounts { cold, unchanged, one_source_moved, both_sources_moved };
}

/// The four claims the counts make, in the order the fixture establishes them.
///
/// Together they are the whole of this test's name. Nothing is charged for a tree that did not
/// move; a source that did move is charged; each moved source is charged the same, so no
/// unmoved source was charged alongside it; and the cold judgement paid for something no edit
/// ever re-derives, which is the whole-workspace half of the store.
///
/// Each is a discriminator rather than an example. A provider that kept nothing fails the
/// first, one that wrongly reused a moved source's facts fails the second, one that re-derived
/// every source on any edit fails the third, and one that re-derived the workspace facts on an
/// edit fails the fourth.
fn Assert_Only_The_Moved_Source_Was_Charged(counted: &MaterializationCounts)
{
    assert_eq!(
        counted.unchanged, 0,
        "an unchanged second judgement wrote {} facts against a cold {}: every family proves its \
         fact current before filing since 6bb62542, so an unmoved tree must file nothing at all",
        counted.unchanged, counted.cold
    );
    assert!(
        counted.one_source_moved > 0,
        "moving one source re-derived nothing, so the store is being reused for a subject whose \
         bytes changed -- the diagnostics an editor sees would be the ones from before the edit"
    );
    assert_eq!(
        counted.both_sources_moved,
        counted.one_source_moved.saturating_mul(SOURCES_IN_THE_FIXTURE),
        "moving one of {SOURCES_IN_THE_FIXTURE} sources cost {}, so moving both must cost exactly \
         {SOURCES_IN_THE_FIXTURE} times that and cost {} instead: the two judgements disagree \
         about what one source is worth, which is what charging for an unmoved source looks like",
        counted.one_source_moved,
        counted.both_sources_moved
    );
    assert!(
        counted.cold > counted.both_sources_moved,
        "the cold judgement cost {} and moving every source in the fixture cost {}: a cold run \
         must also pay for the whole-workspace facts, and no edit to a source may re-derive one",
        counted.cold,
        counted.both_sources_moved
    );
}

/// What a provider that kept its state answers is what a provider that never had any
/// answers about the same tree.
///
/// The equivalence `tests/incremental_equivalence.rs` proves over the seam, asserted here
/// over the surface an editor actually calls -- and it is the one falsifier this boundary
/// has for the reassessment cache itself. A cache that wrongly reused a rule whose family
/// moved would still return diagnostics, and they would be the ones from before the edit;
/// the fresh provider is what says they are not.
#[test]
fn Test_A_Reusing_Provider_Should_Answer_What_A_Fresh_One_Answers_Over_The_Same_Tree()
{
    let root = Root("nomos-lsp-provider-clean-recompute-equivalence");

    let answers = Answers_Over(&root);

    let _ignored = std::fs::remove_dir_all(&root);

    assert_eq!(answers.unchanged, answers.cold, "an unchanged tree judged twice must answer the same thing twice");
    assert_ne!(
        answers.after_edit, answers.cold,
        "the edit must change what the provider answers, or this fixture proves nothing \
         about invalidation -- reusing everything and recomputing everything both pass an \
         unchanged comparison"
    );
    assert_eq!(
        answers.after_edit, answers.recomputed,
        "a provider that carried its workspace, store and reassessment cache across the \
         edit answered differently from one that had never judged this tree at all"
    );
}

/// The four judgements this fixture compares, all over the same two-source tree: a reusing
/// provider's cold one, its second over an unchanged tree, its third after `a.rs` gained an
/// item, and one from a provider that had never judged this tree at all.
struct ProviderAnswers
{
    cold: Vec<SourceDiagnostic>,
    unchanged: Vec<SourceDiagnostic>,
    after_edit: Vec<SourceDiagnostic>,
    recomputed: Vec<SourceDiagnostic>,
}

/// Writes the two-source fixture under `root`, then reaches all four of the judgements
/// [`ProviderAnswers`] names -- three through one provider that keeps its state, and one
/// through a provider created for that single call.
fn Answers_Over(root: &std::path::Path) -> ProviderAnswers
{
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("the fresh root above was just created");
    std::fs::write(root.join("b.rs"), "pub fn Fine() {}\n").expect("the fresh root above was just created");

    let mut reusing = NomosDiagnosticProvider::New();
    let cold = reusing.Diagnose(root);
    let unchanged = reusing.Diagnose(root);

    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\npub fn Also_Ok() {}\n").expect("the fresh root above was just created");
    let after_edit = reusing.Diagnose(root);
    let recomputed = NomosDiagnosticProvider::New().Diagnose(root);

    return ProviderAnswers { cold, unchanged, after_edit, recomputed };
}

/// A cold judgement is what the unreassessed path produces, argument for argument.
///
/// [`nomos_check_orchestration::Run`] is the seam `nomos-cli::check` calls, and
/// `Run_Reassessing` is that same pipeline handed a cache. Asserting the two agree on a
/// cold tree is what says this crate changed which entry point it calls and nothing else --
/// not the selection it passes, not the variant, and not the launcher. A wiring mistake
/// there would otherwise show up only as diagnostics an editor renders and no test reads.
#[test]
fn Test_A_Cold_Judgement_Should_Equal_What_The_Unreassessed_Run_Path_Produces()
{
    let root = Root("nomos-lsp-provider-cold-run-equivalence");
    std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("the fresh root above was just created");
    std::fs::write(root.join("b.rs"), "pub fn Fine() {}\n").expect("the fresh root above was just created");

    let judged = NomosDiagnosticProvider::New().Diagnose(&root);
    let expected = Judged_By_Run(&root);

    let _ignored = std::fs::remove_dir_all(&root);

    assert!(
        !expected.is_empty(),
        "the unreassessed path found nothing over this tree, so the comparison below would \
         hold over two empty lists"
    );
    assert_eq!(
        judged, expected,
        "a cold judgement must be what the unreassessed Run path produces over the same tree"
    );
}

/// `root` judged through [`nomos_check_orchestration::Run`], rendered the way
/// [`NomosDiagnosticProvider::Diagnose`] renders its own.
///
/// Spelled out here rather than shared with the provider: a helper both called would
/// compare each against itself, and what is being asserted is that two entry points agree.
fn Judged_By_Run(root: &std::path::Path) -> Vec<SourceDiagnostic>
{
    let sources = Walked_Sources(root).expect("a directory that was just written walks");

    let nomos_check_orchestration::CheckOutcome::Judged { findings, supporting_facts, .. } = Run_Over(root, &sources)
    else
    {
        panic!("a walked tree with two readable sources must reach a judgement");
    };

    let architecture = nomos_repo_policy::architecture::Discover_Workspace(root, &FILE_SYSTEM).unwrap_or_default();
    let assessments = Committed_Assessments(root);
    let context = WalkContext { architecture: &architecture, trail: &supporting_facts, assessments: &assessments };

    return findings.iter().flat_map(|finding| return Diagnostics_For(&context, finding)).collect();
}

/// The outcome [`nomos_check_orchestration::Run`] reaches over `sources` under `root`, with a
/// workspace and a fact store that live only for this call -- the unreassessed path, whose whole
/// difference from [`NomosDiagnosticProvider`] is that it keeps neither between calls.
fn Run_Over(root: &std::path::Path, sources: &[SourceFile]) -> nomos_check_orchestration::CheckOutcome
{
    let mut workspace = None;
    let mut store = MemoryFactStore::New();

    return nomos_check_orchestration::Run(
        sources,
        nomos_check_orchestration::RunContext {
            variant: Host_Variant(),
            root,
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            workspace: &mut workspace,
            store: &mut store,
        },
        &[],
    );
}

/// The requirement identifier the fixture registry below assesses. Any real identifier would
/// do; what matters is that nothing but that registry could have put it in a diagnostic.
const FIXTURE_REQUIREMENT: &str = "AGT-001";

/// What a judged diagnostic carries for the two answers that are read from outside this crate.
///
/// Both are threaded through [`WalkContext`], and a mistake in that threading has exactly one
/// symptom: a payload that is still well-formed and still says something, but says it about an
/// empty declaration. An empty registry answers `[]` and a fresh trail answers `Unrecorded` --
/// so a `Diagnose` that built its context from neither would look identical to one that read a
/// repository declaring nothing. This is the falsifier for that, over a real run: the registry
/// is real, the trail is the one the run returned, and neither answer is reachable from the
/// empty version of its input.
#[test]
fn Test_A_Judged_Diagnostic_Should_Carry_The_Declarations_Of_The_Repository_Under_Check()
{
    let root = Root("nomos-lsp-provider-walk-outward-declarations");
    Write_Fixture_Declaring_A_Requirement(&root);

    let judged = NomosDiagnosticProvider::New().Diagnose(&root);

    let _ignored = std::fs::remove_dir_all(&root);

    let walked: Vec<serde_json::Value> = judged
        .iter()
        .filter_map(|diagnostic| return diagnostic.detail.as_ref())
        .map(|detail| return serde_json::from_str(detail).expect("walk-outward data is a document"))
        .collect();
    assert!(!walked.is_empty(), "the fixture produced no diagnostic carrying walk-outward data, so nothing below is about a real judgement");

    Assert_A_Requirement_Link_Reached_The_Editor(&walked);
    Assert_A_Real_Read_Trail_Reached_The_Editor(&walked);
}

/// Writes a fixture tree under `root` holding two real violations and one committed assessment
/// declaring the rule that reports the first.
///
/// Two sources, because the two violations are judged by rules of different kinds and the pair
/// is the point: `a.rs` carries trailing whitespace, whose rule judges source text and can
/// therefore never have read a fact, while `b.rs` carries a naming violation whose rule does
/// read one. One batch then shows both answers, which no single-source fixture can.
///
/// The entry is written in the registry's own grammar, at the path
/// `nomos_cap_requirement_trace::REGISTRY` names, so what is exercised is the reader this
/// provider actually calls rather than a shape constructed in memory.
fn Write_Fixture_Declaring_A_Requirement(root: &std::path::Path)
{
    std::fs::write(root.join("a.rs"), "pub fn Ok() -> u32 \n{\n    return 1;\n}\n").expect("the fresh root above was just created");
    std::fs::write(root.join("b.rs"), "pub fn poorly_named_export() -> u32\n{\n    let BadLocal = 1;\n    return BadLocal;\n}\n")
        .expect("the fresh root above was just created");

    let registry = root.join(nomos_cap_requirement_trace::REGISTRY);
    std::fs::create_dir_all(&registry).expect("the fresh root above was just created");
    let entry = format!("verdict: Met\nsite: a.rs#Ok\nrule: {}\n", nomos_rules::NO_TRAILING_WHITESPACE);
    std::fs::write(registry.join(format!("{FIXTURE_REQUIREMENT}.assessment")), entry).expect("the registry directory was just created");
}

/// The fixture's one declared link reached a diagnostic, carrying the entry's own verdict.
fn Assert_A_Requirement_Link_Reached_The_Editor(walked: &[serde_json::Value])
{
    let declared: Vec<&serde_json::Value> = walked
        .iter()
        .filter_map(|document| return document.get("requirements"))
        .filter_map(serde_json::Value::as_array)
        .flatten()
        .filter(|link| return link.get("requirement").and_then(serde_json::Value::as_str) == Some(FIXTURE_REQUIREMENT))
        .collect();

    assert!(
        !declared.is_empty(),
        "no diagnostic reached {FIXTURE_REQUIREMENT}, so the committed registry under this root was never read: {walked:?}"
    );
    for link in declared
    {
        assert_eq!(
            link.get("verdict").and_then(serde_json::Value::as_str),
            Some("Met"),
            "the link must carry the entry's own verdict, not one reached here: {link}"
        );
    }
}

/// One batch carried both a trail the run really recorded and a structural absence, kept apart.
///
/// `Unrecorded` is what a fresh trail answers for every fact-reading rule, and `NotFactBacked`
/// is answered from the descriptor whether or not a trail exists -- so neither on its own would
/// fail if the context had been built from `SupportingFactTrail::New()`. A `Read` answer cannot
/// be produced that way at all, which is why it is asserted first; the second assertion is what
/// keeps the structural answer from quietly becoming the only one a batch ever shows.
fn Assert_A_Real_Read_Trail_Reached_The_Editor(walked: &[serde_json::Value])
{
    let answers: Vec<&str> = walked
        .iter()
        .filter_map(|document| return document.get("supporting_facts"))
        .filter_map(|facts| return facts.get("answer"))
        .filter_map(serde_json::Value::as_str)
        .collect();

    assert!(
        answers.contains(&"Read"),
        "no diagnostic carried a trail this run recorded, so the context was built from an empty one: {answers:?}"
    );
    assert!(
        answers.contains(&"NotFactBacked"),
        "the fixture's source-text violation lost its structural answer, so one batch no longer shows two of the four shapes: {answers:?}"
    );

    let tuples: usize = walked
        .iter()
        .filter_map(|document| return document.get("supporting_facts"))
        .filter(|facts| return facts.get("answer").and_then(serde_json::Value::as_str) == Some("Read"))
        .filter_map(|facts| return facts.get("reads"))
        .filter_map(serde_json::Value::as_array)
        .map(Vec::len)
        .sum();
    assert!(tuples > 0, "a `Read` answer reached the editor carrying no tuples at all, which is not what this run observed: {walked:?}");
}
