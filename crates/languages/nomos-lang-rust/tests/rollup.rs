//! The dependent half of invalidation, driven by the code that ships rather than by a
//! fixture that materializes its own edges.
//!
//! `nomos-analysis` has always had the transitive machinery and has never had a producer:
//! every fact this workspace wrote outside a test was a leaf, so
//! [`InvalidationReport::dependent`] was a field no run could make non-empty and the whole
//! propagation path was reachable only from a test that invented a dependency to feed it.
//! A test that materializes its own edge proves the reverse index works, which was never in
//! doubt; it says nothing about whether anything uses it.
//!
//! So every edge below comes from [`nomos_lang_rust::rollup::Materialize_Index`] observing
//! its own reads. Nothing here writes a dependency array, and the assertions are about what
//! a change travels along rather than about what was declared.

use nomos_analysis::{FactStore, GenerationCause, MemoryFactStore, ReadOutcome};
use nomos_capability::{Registry, Requirement};
use nomos_cap_syntax::PUBLIC;
use nomos_contracts::{
    Assurance, BuildVariantId, ConfigurationId, Digest128, EvidenceClass, FactVariant,
    GenerationId, Guarantee, IncrementalGranularity, SnapshotId, SubjectId,
};
use nomos_lang_rust::rollup::{self, Against, Module, ModuleMember, Outcome, Rolled};
use nomos_lang_rust::{FactContext, Materialization};
use nomos_model::Content_Digest;

const ALPHA: &str = "pub fn Alpha() {}\nfn hidden() {}\n";
const BETA: &str = "pub struct Beta;\npub mod inner { pub fn Deep() {} }\n";

fn Subject(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
}

fn Context(generation: GenerationId) -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
        generation,
    };
}

/// A registry holding both agreements: the one this crate offers a leaf against, and the
/// one it offers a rollup against.
fn Registry_With_Both() -> Registry
{
    let mut registry = Registry::New();

    registry
        .Declare(nomos_cap_syntax::Capability_Contract())
        .expect("the syntax contract is declared once");
    registry
        .Offer(nomos_lang_rust::Provider_Offer())
        .expect("the parser's offer is within the syntax ceiling");

    registry
        .Declare(rollup::Capability_Contract())
        .expect("the module-index contract is declared once");
    registry
        .Offer(rollup::Provider_Offer())
        .expect("the rollup's offer is within its own ceiling");

    return registry;
}

/// What a caller asks of the syntax capability. Handed to the rollup rather than composed
/// by it, which is what keeps the key it rebuilds the same key the run wrote.
fn Need() -> Requirement
{
    return Requirement::New(
        nomos_cap_syntax::Capability(),
        nomos_cap_syntax::CONTRACT_VERSION,
        Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        ),
    );
}

/// Writes one file's syntax fact and hands back the key it was filed under.
fn Materialize_Leaf(
    store: &mut MemoryFactStore,
    path: &str,
    source: &str,
    context: FactContext,
) -> nomos_analysis::FactKey
{
    let Materialization::Materialized(fact) = nomos_lang_rust::Materialize(Subject(path), source, context)
    else
    {
        panic!("`{path}` parses");
    };

    let key = fact.Key().clone();
    store
        .Materialize(*fact, &[])
        .expect("a leaf is never written behind the generation it names");

    return key;
}

/// The whole arrangement: two leaves, one rollup over them, in one store.
struct RolledModule
{
    store: MemoryFactStore,
    alpha: nomos_analysis::FactKey,
    beta: nomos_analysis::FactKey,
    rolled: Rolled,
}

fn Roll_Up_Two_Files() -> RolledModule
{
    let mut store = MemoryFactStore::New();
    let registry = Registry_With_Both();
    let context = Context(GenerationId::INITIAL);

    let alpha = Materialize_Leaf(&mut store, "alpha.rs", ALPHA, context);
    let beta = Materialize_Leaf(&mut store, "beta.rs", BETA, context);

    let module = Module {
        // A subject of its own. Sharing one with a member would put the rollup in the
        // direct set of any change to that member, which is the property under test
        // vanishing into the one that was already proven.
        subject: Subject("the/module"),
        members: vec![
            ModuleMember::Of(Subject("alpha.rs"), ALPHA),
            ModuleMember::Of(Subject("beta.rs"), BETA),
        ],
    };

    let rolled = rollup::Materialize_Index(&mut store, &Against { registry: &registry, need: &Need(), context }, &module)
        .expect("the rollup is not written behind the generation it names");

    return RolledModule {
        store,
        alpha,
        beta,
        rolled,
    };
}

/// The claim the item was written for. A change to a member reaches the rollup, and it
/// reaches it *through an edge* — the report says so, in the field nothing could previously
/// fill.
#[test]
fn Test_Editing_A_Member_Should_Reach_The_Rollup_Through_A_Dependency_Edge()
{
    let mut rolled = Roll_Up_Two_Files();
    let next = GenerationId::INITIAL.Next();

    let report = rolled.store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: Subject("alpha.rs"),
            granularity: IncrementalGranularity::File,
        },
        next,
    );

    assert_eq!(
        report.direct,
        vec![rolled.alpha.clone()],
        "the edited file's own fact is what the cause names, and nothing else is"
    );
    assert_eq!(
        report.dependent,
        vec![rolled.rolled.key.clone()],
        "the rollup was not named by the cause and must have been reached through the edge \
         the reader recorded: {report:#?}"
    );
    assert!(
        !report.dependent.is_empty(),
        "the dependent set is the half of invalidation this test exists for"
    );
    assert!(
        !report.direct.contains(&rolled.rolled.key),
        "a rollup reported as directly invalidated is a rollup keyed on one of its own \
         inputs, which would make this test pass while measuring the direct path"
    );
}

/// The negative control. If everything in the store were invalidated by any cause — a
/// broken `Names`, a frontier that swept the whole map — the assertion above would hold for
/// the wrong reason.
#[test]
fn Test_A_Change_To_Nothing_In_The_Module_Should_Reach_Neither()
{
    let mut rolled = Roll_Up_Two_Files();

    let report = rolled.store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: Subject("elsewhere.rs"),
            granularity: IncrementalGranularity::File,
        },
        GenerationId::INITIAL.Next(),
    );

    assert!(
        report.direct.is_empty() && report.dependent.is_empty(),
        "a file no member is and no rollup reads invalidated something: {report:#?}"
    );
    assert_eq!(
        report.retained, 3,
        "two leaves and one rollup are still live"
    );
}

/// The edges are what the reader saw, and the store gives them back.
#[test]
fn Test_The_Store_Should_Return_The_Edges_The_Reader_Recorded()
{
    let rolled = Roll_Up_Two_Files();

    let held = rolled.store.Dependencies_Of(&rolled.rolled.key);

    assert_eq!(
        held, rolled.rolled.dependencies,
        "what was declared at materialization is what the store holds"
    );
    assert_eq!(held.len(), 2, "one edge per member read: {held:#?}");

    let read: Vec<&nomos_analysis::FactKey> = held
        .iter()
        .filter(|dependency| return dependency.outcome == ReadOutcome::Materialized)
        .map(|dependency| return &dependency.key)
        .collect();

    assert!(
        read.contains(&&rolled.alpha) && read.contains(&&rolled.beta),
        "both members' facts were read and both must be edges: {held:#?}"
    );
}

/// A member with nothing computed for it is still an edge.
///
/// This is the case the comment in `nomos-analysis`'s reader has named since it was
/// written and nothing exercised: a read that found nothing is a real read, and the day the
/// parser does have an answer the rollup is stale. Without the edge, nothing would know.
#[test]
fn Test_A_Member_With_No_Fact_Should_Still_Be_An_Edge()
{
    let mut store = MemoryFactStore::New();
    let registry = Registry_With_Both();
    let context = Context(GenerationId::INITIAL);

    Materialize_Leaf(&mut store, "alpha.rs", ALPHA, context);

    let module = Module {
        subject: Subject("the/module"),
        members: vec![
            ModuleMember::Of(Subject("alpha.rs"), ALPHA),
            // Never materialized. The rollup is asked for it anyway, which is what a real
            // run does when a provider refused a file.
            ModuleMember::Of(Subject("missing.rs"), "pub fn Absent() {}\n"),
        ],
    };

    let rolled = rollup::Materialize_Index(&mut store, &Against { registry: &registry, need: &Need(), context }, &module)
        .expect("materializes");

    assert_eq!(rolled.index.Unreachable(), 1, "{:#?}", rolled.index.members);
    assert_eq!(rolled.index.Answered(), 1);
    assert_eq!(
        rolled.dependencies.len(),
        2,
        "the member that answered and the member that did not are both edges: {:#?}",
        rolled.dependencies
    );

    let missed = rolled
        .dependencies
        .iter()
        .find(|dependency| return dependency.key.subject == Subject("missing.rs"))
        .expect("the read that found nothing was recorded");
    assert_ne!(
        missed.outcome,
        ReadOutcome::Materialized,
        "an edge to a fact that does not exist must not read as a materialized one"
    );
}

/// What the index is for: an item is attributed to the file that declared it.
///
/// The syntax capability cannot express this — its records carry a name qualified by
/// nesting inside one file and no field for which file — which is why a union of syntax
/// payloads is not this fact and why the second capability exists.
#[test]
fn Test_Every_Entry_Should_Name_The_Member_That_Declared_It()
{
    let rolled = Roll_Up_Two_Files();
    let index = &rolled.rolled.index;

    assert_eq!(index.Answered(), 2);
    assert_eq!(index.Unreachable(), 0);
    assert_eq!(index.Approximated(), 0);

    let alpha_entries: Vec<&str> = index
        .items
        .iter()
        .filter(|entry| return entry.member == Subject("alpha.rs"))
        .map(|entry| return entry.qualified_name.as_str())
        .collect();
    let beta_entries: Vec<&str> = index
        .items
        .iter()
        .filter(|entry| return entry.member == Subject("beta.rs"))
        .map(|entry| return entry.qualified_name.as_str())
        .collect();

    assert_eq!(alpha_entries, vec!["Alpha", "hidden"]);
    assert_eq!(beta_entries, vec!["Beta", "inner", "inner::Deep"]);

    let public = index
        .items
        .iter()
        .filter(|entry| return entry.visibility == PUBLIC)
        .count();
    assert_eq!(public, 4, "{:#?}", index.items);
}

/// A derivation is no stronger than what it derived from, on every axis and in the
/// evidence class.
#[test]
fn Test_The_Derived_Fact_Should_Claim_No_More_Than_Its_Inputs()
{
    let rolled = Roll_Up_Two_Files();

    let held = rolled
        .store
        .Current(
            &rolled.rolled.key.clone().At(GenerationId::INITIAL),
            GenerationId::INITIAL,
        )
        .expect("the rollup was written and is current");

    assert_eq!(held.evidence, EvidenceClass::Derived);
    assert_eq!(held.payload.schema, rollup::Payload_Schema());

    let leaf = nomos_lang_rust::Declared_Guarantee();
    let derived = held.guarantee;

    assert_eq!(derived.variant, leaf.variant);
    assert_eq!(derived.completeness, leaf.completeness);
    // Stated with the type's own operation rather than with an ordering comparison, because
    // coarser is *lower* on this enum and a `>=` written the intuitive way asserts the
    // opposite of what it reads as.
    assert_eq!(
        leaf.incremental.Broadened_To(derived.incremental),
        derived.incremental,
        "a rollup cannot refresh more finely than the leaves it is over: {derived:?} against \
         {leaf:?}"
    );
}

/// The cost of `Project` granularity, recorded rather than absorbed.
///
/// A file-granular cause cannot refresh part of a rollup, so the engine broadens it — and
/// says so. If this stopped being reported, a run would be refreshing a whole module for a
/// one-file edit with nothing in the report to show it.
#[test]
fn Test_The_Rollup_Should_Broaden_A_File_Granular_Cause()
{
    let mut rolled = Roll_Up_Two_Files();

    let report = rolled.store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: Subject("alpha.rs"),
            granularity: IncrementalGranularity::File,
        },
        GenerationId::INITIAL.Next(),
    );

    let broadened: Vec<&nomos_analysis::FactKey> = report
        .broadened
        .iter()
        .map(|broadening| return &broadening.key)
        .collect();

    assert_eq!(
        broadened,
        vec![&rolled.rolled.key],
        "the rollup is the only thing here that cannot be refreshed at file granularity: \
         {report:#?}"
    );
    assert!(report.Report().contains("1 through dependency edges"));
}

/// The same module described two ways is one fact, because the member order is the
/// provider's and not the caller's.
#[test]
fn Test_The_Member_Order_Should_Not_Change_The_Fact()
{
    let context = Context(GenerationId::INITIAL);
    let alpha = ModuleMember::Of(Subject("alpha.rs"), ALPHA);
    let beta = ModuleMember::Of(Subject("beta.rs"), BETA);

    let forwards = rollup::Index_Key(Subject("the/module"), &[alpha, beta], context);
    let backwards = rollup::Index_Key(Subject("the/module"), &[beta, alpha], context);
    let doubled = rollup::Index_Key(Subject("the/module"), &[alpha, beta, alpha], context);

    assert_eq!(forwards.Digest(), backwards.Digest());
    assert_eq!(
        forwards.Digest(),
        doubled.Digest(),
        "a module is a set of files, so naming one twice does not make it two members"
    );
}

/// The negative control for the key. A module whose members changed must not keep its
/// address, or a stale rollup stays readable as a current one.
#[test]
fn Test_A_Module_Whose_Members_Changed_Should_Not_Keep_Its_Key()
{
    let context = Context(GenerationId::INITIAL);
    let alpha = ModuleMember::Of(Subject("alpha.rs"), ALPHA);
    let beta = ModuleMember::Of(Subject("beta.rs"), BETA);
    let edited = ModuleMember::Of(Subject("alpha.rs"), "pub fn Alpha() {}\npub fn Added() {}\n");
    // Same bytes as beta, different file.
    let renamed = ModuleMember::Of(Subject("gamma.rs"), BETA);

    let original = rollup::Index_Key(Subject("the/module"), &[alpha, beta], context).Digest();

    assert_ne!(
        original,
        rollup::Index_Key(Subject("the/module"), &[edited, beta], context).Digest(),
        "editing a member must re-address the rollup"
    );
    assert_ne!(
        original,
        rollup::Index_Key(Subject("the/module"), &[alpha], context).Digest(),
        "dropping a member must re-address the rollup"
    );
    assert_ne!(
        original,
        rollup::Index_Key(Subject("the/module"), &[alpha, renamed], context).Digest(),
        "a module is which files it has, not only what they contain"
    );
}

/// The rollup's own answer survives being written and read back through its schema.
#[test]
fn Test_The_Payload_Should_Decode_Under_This_Schemas_Own_Reader()
{
    let rolled = Roll_Up_Two_Files();

    let held = rolled
        .store
        .Current(
            &rolled.rolled.key.clone().At(GenerationId::INITIAL),
            GenerationId::INITIAL,
        )
        .expect("the rollup is current");

    let decoded = rollup::Parse_Index(&held.payload.bytes).expect("this provider writes its schema");

    assert_eq!(decoded, rolled.rolled.index);
    assert_eq!(decoded.module, Subject("the/module"));
    assert!(
        decoded
            .members
            .iter()
            .all(|member| return member.outcome == Outcome::Read)
    );
}
