//! What each rule actually read, against what its descriptor declares it requires.
//!
//! `RuleDescriptor::requires` is a declared universe and, until this file existed, nothing
//! anywhere compared it against the reality it claims to enumerate -- six sites read it, one
//! of them publishing it to editors, and the three tests constraining it all ask whether a
//! variant maps to a capability rather than whether a rule reads what it says it reads.
//! `OD-HOST-016`'s decision 8 named that comparison as the thing the read trail makes
//! possible for the first time, and `OD-COMPLETENESS-001` is the shape it is an instance of.
//!
//! Two obligations live here and they are different. [`Test_Every_Selected_Rules_Reads_
//! Should_Be_Declared_By_Its_Descriptor`] is the declared-versus-observed comparison. The
//! rest are the per-rule reader's own falsifier: a trail collected on one reader shared by
//! every rule would carry each rule's reads forward into the next one's answer, and
//! [`Test_A_Later_Rules_Trail_Should_Not_Contain_An_Earlier_Rules_Reads`] is what a shared
//! reader fails.

use nomos_analysis::{MemoryFactStore, ReadOutcome};
use nomos_contracts::{CapabilityId, RuleId};
use nomos_model::Subject_Of_Path;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::{RequiredFact, RuleDescriptor, SourceFile, DESCRIPTORS};
use nomos_workspace::Workspace;

use crate::{CheckOutcome, RuleReassessmentCache, RunContext, Run_Reassessing, SupportingFactTrail, SupportingFacts};

use super::{Repository_Root, Source_File, SourceText, Test_Variant};

/// The capability families whose materialization runs a subprocess over the whole
/// workspace -- `cargo metadata`, `cargo clippy` and `cargo deny`.
///
/// Excluded from the selection below for the reason `tests::composition`'s own
/// `Architectural_Rules` already states for the same three: their answers depend on the
/// ambient state of the real repository at the moment the test runs, and minutes of
/// subprocess time buys nothing this file is asking about. Every other family is a pure
/// function over bytes, so the selection derived from this exclusion is wide -- most of
/// `DESCRIPTORS` -- rather than a handful named by hand.
const SUBPROCESS_FAMILIES: [RequiredFact; 3] = [RequiredFact::DependencyEdges, RequiredFact::LintDiagnostics, RequiredFact::DependencyPolicy];

/// Every composed rule the run below judges: all of them but the ones a subprocess family
/// would drag in.
///
/// Derived from `DESCRIPTORS` rather than listed, so a rule added later joins this
/// comparison with no second edit -- which is the whole complaint `requires` has been open
/// to, one level up.
fn Selected_Rules() -> Vec<RuleId>
{
    return DESCRIPTORS
        .iter()
        .filter(|descriptor| return !descriptor.requires.iter().any(|required| return SUBPROCESS_FAMILIES.contains(required)))
        .map(RuleDescriptor::Rule)
        .collect();
}

/// Whether `rule` is one the run below judges.
fn Is_Selected(rule: &str) -> bool
{
    return Selected_Rules().iter().any(|id| return id.As_Str() == rule);
}

/// The one source every run here is judged over.
fn Fixture() -> [SourceFile; 1]
{
    return [Source_File("a.rs", SourceText("pub fn Ok() {}\n"))];
}

/// One real run, reading the trail back off the outcome it returned.
///
/// Off the outcome rather than off the cache that recorded it, because the outcome is where
/// `OD-HOST-016`'s decision 4 put the answer; reading it from the cache would prove something
/// weaker than what every test below claims.
fn Observed() -> SupportingFactTrail
{
    return Judged_Trail(&Fixture(), &mut None, &mut MemoryFactStore::New(), &mut RuleReassessmentCache::New());
}

/// The trail `Run_Reassessing` carried out on its own outcome for one call over `sources`.
fn Judged_Trail(
    sources: &[SourceFile], workspace: &mut Option<Workspace>, store: &mut MemoryFactStore, cache: &mut RuleReassessmentCache,
) -> SupportingFactTrail
{
    let outcome = Run_Reassessing(
        sources,
        RunContext {
            variant: Test_Variant(),
            root: &Repository_Root(),
            launcher: &StdProgramLauncher,
            filesystem: &StdFileSystem,
            environment: &StdEnvironment,
            workspace,
            store,
        },
        &Selected_Rules(),
        cache,
    );

    let CheckOutcome::Judged { supporting_facts, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };

    return supporting_facts;
}

/// The capabilities `descriptor` declares it requires.
fn Declared(descriptor: &RuleDescriptor) -> Vec<CapabilityId>
{
    return descriptor.requires.iter().map(|required| return required.Capability()).collect();
}

/// The capabilities `rule`'s trail says it actually read, in the order the reduction holds
/// them.
fn Observed_Capabilities(trail: &SupportingFactTrail, rule: &str) -> Vec<CapabilityId>
{
    return trail
        .Observed_Reads(rule)
        .unwrap_or_default()
        .iter()
        .map(|read| return read.capability.clone())
        .collect();
}

/// Every rule in the selection whose observed reads name a capability its own descriptor does
/// not declare, as a line a person can act on.
fn Disagreements(trail: &SupportingFactTrail) -> Vec<String>
{
    let mut disagreements = Vec::new();
    for descriptor in DESCRIPTORS.iter().filter(|descriptor| return Is_Selected(descriptor.id))
    {
        let declared = Declared(descriptor);
        for capability in Observed_Capabilities(trail, descriptor.id).iter().filter(|capability| return !declared.contains(capability))
        {
            disagreements.push(format!("`{}` read `{capability:?}`, declaring {declared:?}", descriptor.id));
        }
    }

    return disagreements;
}

/// The comparison `requires` has never had: what a rule read against what it says it needs.
///
/// Reported rather than repaired. A descriptor is a declaration somebody made, and a
/// disagreement between it and reality is a decision to take, not a line to edit until the
/// assertion passes.
#[test]
fn Test_Every_Selected_Rules_Reads_Should_Be_Declared_By_Its_Descriptor()
{
    let trail = Observed();

    let disagreements = Disagreements(&trail);

    assert!(disagreements.is_empty(), "a rule read a capability its descriptor does not declare: {disagreements:?}");
}

/// The per-rule reader's falsifier.
///
/// One reader per run accumulates every rule's reads onto one trail, so a later rule's answer
/// would contain an earlier rule's capability. `GUARANTEE_DECLARES_ITS_EXERCISER` runs after
/// `NAMING_CONVENTION` and declares only the syntax family, so the naming policy appearing in
/// its trail is exactly the leak a shared reader produces.
#[test]
fn Test_A_Later_Rules_Trail_Should_Not_Contain_An_Earlier_Rules_Reads()
{
    let trail = Observed();
    let naming_policy = nomos_cap_naming_policy::Capability();

    let earlier = Observed_Capabilities(&trail, nomos_rules::NAMING_CONVENTION);
    let later = Observed_Capabilities(&trail, nomos_rules::GUARANTEE_DECLARES_ITS_EXERCISER);

    assert!(earlier.contains(&naming_policy), "the earlier rule must really read the policy, or this proves nothing: {earlier:?}");
    assert!(!later.contains(&naming_policy), "the later rule's trail carries a read it never made: {later:?}");
}

/// A rule that reads a fact answers with what it read, and the answer carries the provenance
/// the store resolved rather than a digest a reader cannot open.
#[test]
fn Test_A_Fact_Reading_Rule_Should_Answer_With_What_It_Read()
{
    let trail = Observed();

    let answer = trail.Facts_For(nomos_rules::GUARANTEE_DECLARES_ITS_EXERCISER);

    let SupportingFacts::Read(reads) = answer
    else
    {
        panic!("a rule that ran and read the syntax family must answer with its reads: {answer:?}");
    };
    let answered = reads.iter().find(|read| return read.outcome == ReadOutcome::Materialized).expect("the syntax fact this run materialized answered");
    assert!(answered.guarantee.is_some(), "an answered read carries the guarantee of the offer that answered it");
    assert!(answered.evidence.is_some(), "an answered read carries the evidence class of the fact that answered it");
}

/// The first of the four shapes: structural, derived from the descriptor, and never from the
/// rule having happened to read nothing.
#[test]
fn Test_A_Source_Text_Rule_Should_Answer_Not_Fact_Backed()
{
    let trail = Observed();

    assert_eq!(trail.Facts_For(nomos_rules::NO_TRAILING_WHITESPACE), SupportingFacts::NotFactBacked);
    assert_eq!(
        trail.Observed_Reads(nomos_rules::NO_TRAILING_WHITESPACE),
        Some([].as_slice()),
        "a source-text rule is handed a reader it discards, so its trail is what proves the shape is structural"
    );
}

/// The fourth shape: a rule this run never reached is unrecorded, which is a different claim
/// from having judged without evidence.
#[test]
fn Test_A_Rule_The_Run_Never_Reached_Should_Answer_Unrecorded()
{
    let trail = Observed();

    assert_eq!(trail.Facts_For(nomos_rules::LINT_DIAGNOSTICS), SupportingFacts::Unrecorded);
}

/// The third shape, and what it exists to stop: a finding raised while a capability was
/// materialized carries a rule that never judged, and must not read as that rule having
/// judged without evidence.
#[test]
fn Test_A_Materialization_Raised_Rule_Should_Not_Answer_Unrecorded()
{
    let mut trail = SupportingFactTrail::New();

    trail.Note_Materialization_Raised(&[Unjudgeable_Finding()]);

    assert_eq!(trail.Facts_For(nomos_rules::LINT_DIAGNOSTICS), SupportingFacts::RaisedByMaterialization);
}

/// One finding of the shape a capability's own materialization raises: it names a rule and
/// never passed through that rule's judgment.
fn Unjudgeable_Finding() -> nomos_contracts::Finding
{
    return nomos_contracts::Finding {
        address: None,
        rule: RuleId::New(nomos_rules::LINT_DIAGNOSTICS),
        subject: Subject_Of_Path("a.rs"),
        subject_name: "a.rs".to_owned(),
        applicability: nomos_contracts::Applicability::MissingCapability,
        evidence: nomos_contracts::EvidenceClass::Derived,
        gate: nomos_contracts::GateCategory::Advisory,
        summary: "no provider offered the capability this rule requires".to_owned(),
        locations: vec!["a.rs".to_owned()],
    };
}

/// The comparison above is not vacuous: the selection really is most of the composed table,
/// and the three rules the tests below name by hand are in it.
#[test]
fn Test_The_Selection_Should_Be_Most_Of_The_Composed_Table()
{
    let selected = Selected_Rules();

    assert!(selected.len() * 2 > DESCRIPTORS.len(), "{} of {}", selected.len(), DESCRIPTORS.len());
    for rule in [nomos_rules::NAMING_CONVENTION, nomos_rules::GUARANTEE_DECLARES_ITS_EXERCISER, nomos_rules::NO_TRAILING_WHITESPACE]
    {
        assert!(Is_Selected(rule), "{rule}");
    }
}

/// Decision 4's second half: the trail is cached beside the findings it belongs to, so a
/// caller that reuses findings across calls is not handed an empty answer for the findings it
/// actually shows.
///
/// The second call over an unmoved tree re-runs no rule at all -- every one is served from
/// `RuleReassessmentCache` -- so a trail that lived only for the call would come back empty
/// here while the findings it belongs to came back in full. That is the exact failure this
/// asserts against, and it is the state an editor session spends nearly every keystroke in.
#[test]
fn Test_A_Reused_Call_Should_Still_Carry_The_Trail_Of_The_Call_That_Judged()
{
    let sources = Fixture();
    let mut workspace = None;
    let mut store = MemoryFactStore::New();
    let mut cache = RuleReassessmentCache::New();

    let judged = Judged_Trail(&sources, &mut workspace, &mut store, &mut cache);
    let recorded = cache.Recorded();
    let reused = Judged_Trail(&sources, &mut workspace, &mut store, &mut cache);

    assert_eq!(cache.Recorded(), recorded, "the second call over an unmoved tree must re-run no rule, or this proves nothing");
    assert_eq!(reused, judged, "the reused call's outcome must carry the trail of the call that actually judged");
    assert!(
        matches!(reused.Facts_For(nomos_rules::NAMING_CONVENTION), SupportingFacts::Read(reads) if !reads.is_empty()),
        "a reused finding's rule still answers with what it read"
    );
}
