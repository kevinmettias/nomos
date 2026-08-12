use nomos_analysis::{
    Component, Context, Dependency, FactKey, FactPayload, FactReader, FactStore, GenerationCause,
    GuaranteeDigest, InputDigest, InvalidationReport, MaterializedFact, MemoryFactStore,
    ReadOutcome, Reader,
};
use nomos_capability::{CapabilityContract, ProviderOffer, Registry, Requirement};
use nomos_contracts::{
    Applicability, Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion,
    Digest128, EvidenceClass, FactVariant, GenerationId, Guarantee, IncrementalGranularity,
    ProviderId, SchemaId, SnapshotId, SubjectId,
};
use std::collections::BTreeSet;

const SYNTAX: &str = "nomos.cap.syntax.tree";

const SEMANTIC: &str = "nomos.cap.semantic.resolution";

fn Digest(seed: u8) -> Digest128
{
    return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
}

fn Subject(seed: u8) -> SubjectId
{
    return SubjectId::From_Digest(Digest(seed));
}

fn Snapshot(seed: u8) -> SnapshotId
{
    return SnapshotId::From_Digest(Digest(seed));
}

fn Variant(seed: u8) -> BuildVariantId
{
    return BuildVariantId::From_Digest(Digest(seed));
}

fn Configuration(seed: u8) -> ConfigurationId
{
    return ConfigurationId::From_Digest(Digest(seed));
}

fn Syntactic() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
}

fn Coarse() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::WholeWorkspace,
    );
}

fn Base() -> FactKey
{
    return FactKey {
        contract: CapabilityId::New(SYNTAX),
        contract_version: ContractVersion::New(1, 0),
        subject: Subject(1),
        semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
        provider: ProviderId::New("nomos.provider.rust-syntax"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Syntactic()),
        variant: Variant(3),
        configuration: Configuration(4),
    };
}

fn Varied(component: Component) -> FactKey
{
    let mut key = Base();
    match component
    {
        Component::Contract => key.contract = CapabilityId::New(SEMANTIC),
        Component::ContractVersion => key.contract_version = ContractVersion::New(2, 0),
        Component::Subject => key.subject = Subject(9),
        Component::SemanticInputs => key.semantic_inputs = InputDigest::Of(&[b"fn main() { x }"]),
        Component::Provider => key.provider = ProviderId::New("nomos.provider.other"),
        Component::ProviderVersion => key.provider_version = ContractVersion::New(1, 1),
        Component::Guarantee => key.guarantee = GuaranteeDigest::Of(&Coarse()),
        Component::Variant => key.variant = Variant(9),
        Component::Configuration => key.configuration = Configuration(9),
    }

    return key;
}

fn Fact(key: &FactKey, generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(generation),
        snapshot: Snapshot(2),
        evidence: EvidenceClass::Derived,
        guarantee: Syntactic(),
        payload: FactPayload::New(SchemaId::New("nomos.syntax.v1"), b"tree".to_vec()),
    };
}

fn Stored(key: &FactKey) -> MemoryFactStore
{
    let mut store = MemoryFactStore::New();
    let fact = Fact(key, GenerationId::INITIAL);
    store.Materialize(fact, &[]).expect("materializes");

    return store;
}

/// The same fact, read from a named tree.
fn Fact_From(key: &FactKey, snapshot: SnapshotId) -> MaterializedFact
{
    let mut fact = Fact(key, GenerationId::INITIAL);
    fact.snapshot = snapshot;

    return fact;
}

/// A fact for `key` that reached for `upstream` and got the given outcome.
fn Materialize_Reading(
    store: &mut MemoryFactStore,
    key: &FactKey,
    upstream: &FactKey,
    outcome: ReadOutcome,
)
{
    let fact = Fact(key, GenerationId::INITIAL);
    let edge = Dependency {
        key: upstream.clone(),
        outcome,
    };

    store.Materialize(fact, &[edge]).expect("materializes");
}

/// What a change to one subject invalidates, at file granularity.
fn Subject_Changed(store: &mut MemoryFactStore, subject: SubjectId, next: GenerationId)
    -> InvalidationReport
{
    return store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject,
            granularity: IncrementalGranularity::File,
        },
        next,
    );
}

/// What a checkout invalidates, naming the members that differ.
fn Snapshot_Replaced(
    store: &mut MemoryFactStore,
    differing: BTreeSet<SubjectId>,
    next: GenerationId,
) -> InvalidationReport
{
    return store.Invalidate(
        &GenerationCause::SnapshotReplaced {
            from: Snapshot(2),
            to: Snapshot(9),
            differing,
        },
        next,
    );
}

fn Context_At(generation: GenerationId) -> Context
{
    return Context {
        snapshot: Snapshot(2),
        variant: Variant(3),
        configuration: Configuration(4),
        generation,
    };
}

fn Offering(guarantee: Guarantee) -> Registry
{
    let mut registry = Registry::New();
    registry
        .Declare(CapabilityContract {
            id: CapabilityId::New(SYNTAX),
            version: ContractVersion::New(1, 0),
            summary: "A syntax tree".to_owned(),
            ceiling: Guarantee::New(
                FactVariant::SemanticallyResolved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::Symbol,
            ),
        })
        .expect("declares");
    registry
        .Offer(ProviderOffer {
            provider: ProviderId::New("nomos.provider.rust-syntax"),
            capability: CapabilityId::New(SYNTAX),
            version: ContractVersion::New(1, 0),
            guarantee,
        })
        .expect("offers");

    return registry;
}

fn Needing(guarantee: Guarantee) -> Requirement
{
    let capability = CapabilityId::New(SYNTAX);
    let version = ContractVersion::New(1, 0);

    return Requirement::New(capability, version, guarantee);
}

#[test]
fn Test_The_Component_List_Should_Cover_Every_Part_Of_The_Key()
{
    assert_eq!(
        Base().Parts().len(),
        Component::All().len(),
        "a component of the key is not enumerated, so nothing varies it and nothing tests it"
    );
}

#[test]
fn Test_Every_Component_Should_Reach_The_Key_Digest()
{
    let base = Base().Digest();

    for component in Component::All()
    {
        assert_ne!(
            Varied(*component).Digest(),
            base,
            "{} does not reach the digest, so a fact would be reused under a key that does \
             not describe it",
            component.Label()
        );
    }
}

#[test]
fn Test_A_Fact_Should_Answer_Its_Own_Key()
{
    let key = Base();
    let store = Stored(&key);

    assert!(
        store
            .Current(&key.clone().At(GenerationId::INITIAL), GenerationId::INITIAL)
            .is_some(),
        "the store answers nothing at all, so a miss proves nothing"
    );
}

#[test]
fn Test_A_Change_In_Any_One_Component_Should_Miss_The_Cache()
{
    let store = Stored(&Base());

    for component in Component::All()
    {
        let varied = Varied(*component);

        assert!(
            store
                .Current(&varied.At(GenerationId::INITIAL), GenerationId::INITIAL)
                .is_none(),
            "a fact answered a question that differs in {}, which is a cached answer to a \
             different question",
            component.Label()
        );
    }
}

#[test]
fn Test_Two_Components_Should_Not_Cancel_Each_Other_Out()
{
    let mut both = Base();
    both.subject = Subject(9);
    both.configuration = Configuration(9);

    assert_ne!(both.Digest(), Base().Digest());
    assert_ne!(both.Digest(), Varied(Component::Subject).Digest());
    assert_ne!(both.Digest(), Varied(Component::Configuration).Digest());
}

/// The property that closed OD-ANALYSIS-001.
///
/// A workspace state is not part of what a fact is. The same subject, the same content and
/// the same provider under the same guarantee is one fact whichever tree it was read from.
/// While a key carried a snapshot, it was two — so a corpus recomputed whenever any one
/// file anywhere in the workspace changed, because a workspace snapshot is a digest over
/// all of its members.
#[test]
fn Test_Two_Workspace_States_Should_Not_Produce_Two_Facts()
{
    let key = Base();
    let mut store = MemoryFactStore::New();
    let measured = Fact_From(&key, Snapshot(2));
    let asked_again = Fact_From(&key, Snapshot(9));

    assert_eq!(
        measured.Key().Digest(),
        asked_again.Key().Digest(),
        "the tree reached the identity, so one file's fact is re-addressed by a change to \
         another file entirely"
    );
    store.Materialize(measured, &[]).expect("materializes");
    let served = store
        .Current(&asked_again.identity, GenerationId::INITIAL)
        .expect("the store holds the answer to the question the second one asks");

    assert_eq!(
        served.snapshot,
        Snapshot(2),
        "and what it serves still names the tree it was read from. A reused fact is not a \
         repeated observation, so its provenance must not be restamped"
    );
}

/// A checkout invalidates what actually differs.
///
/// The cause used to name only the new snapshot and match it against a key component, which
/// meant every fact in the store — the whole corpus, for a checkout that touched one file.
/// It now names the members that differ, which is what the caller doing the replacing
/// already has: two content-addressed maps, and the paths where they disagree.
#[test]
fn Test_Replacing_A_Snapshot_Should_Invalidate_Exactly_The_Members_That_Differ()
{
    let changed = Base();
    let mut untouched = Base();
    untouched.subject = Subject(9);
    let mut store = Stored(&changed);
    let fact = Fact(&untouched, GenerationId::INITIAL);
    store.Materialize(fact, &[]).expect("materializes");
    let next = GenerationId::INITIAL.Next();

    let report = Snapshot_Replaced(&mut store, BTreeSet::from([changed.subject]), next);

    assert_eq!(report.direct, vec![changed]);
    assert!(
        store.Current(&untouched.At(next), next).is_some(),
        "a checkout that touched one file discarded a fact about another"
    );
    assert_eq!(
        report.cause.Granularity(),
        IncrementalGranularity::File,
        "a replacement that names its differing members is a statement about files, and \
         reporting it as WholeWorkspace makes every provider's broadening look unavoidable"
    );
}

/// A replacement that changes no member is visible as one.
///
/// Two states can hold identical members and differ in variant or configuration, and those
/// have causes of their own. What must not happen is that a caller whose diff iterated zero
/// times reads a clean result — the prototype's most repeated defect, one level down.
#[test]
fn Test_A_Replacement_That_Differs_In_Nothing_Should_Say_So()
{
    let key = Base();
    let mut store = Stored(&key);
    let next = GenerationId::INITIAL.Next();

    let report = Snapshot_Replaced(&mut store, BTreeSet::new(), next);

    assert_eq!(report.Invalidated(), 0);
    assert!(
        report.Report().contains("0 member(s) differ"),
        "the report reads as a clean invalidation rather than as an empty one: {}",
        report.Report()
    );
    assert!(
        store.Current(&key.At(next), next).is_some(),
        "nothing differed, so nothing may be discarded"
    );
}

#[test]
fn Test_A_Later_Generation_Should_Still_Answer_The_Same_Key()
{
    let key = Base();
    let store = Stored(&key);

    assert!(
        store
            .Current(&key.At(GenerationId::INITIAL.Next()), GenerationId::INITIAL.Next())
            .is_some(),
        "a fact stopped answering because the workspace moved on, so nothing is ever reused"
    );
}

#[test]
fn Test_The_Same_Question_Twice_Should_Materialize_Once()
{
    let key = Base();
    let mut store = Stored(&key);
    let materializations = store.Materializations();

    let already = store.Current(&key.clone().At(GenerationId::INITIAL), GenerationId::INITIAL);
    if already.is_none()
    {
        let fact = Fact(&key, GenerationId::INITIAL);
        store.Materialize(fact, &[]).expect("materializes");
    }

    assert_eq!(
        store.Materializations(),
        materializations,
        "the second run recomputed a fact the store already held"
    );
}

#[test]
fn Test_A_Backdated_Materialization_Should_Be_Refused()
{
    let key = Base();
    let mut store = MemoryFactStore::New();
    let ahead = Fact(&key, GenerationId::INITIAL.Next());
    let behind = Fact(&key, GenerationId::INITIAL);
    store.Materialize(ahead, &[]).expect("materializes");

    let refusal = store.Materialize(behind, &[]).expect_err("must refuse");

    assert!(format!("{refusal}").contains("behind"), "{refusal}");
}

#[test]
fn Test_A_Changed_Subject_Should_Invalidate_Its_Facts()
{
    let key = Base();
    let mut store = Stored(&key);
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, key.subject, next);

    assert_eq!(report.direct, vec![key.clone()]);
    assert!(store.Current(&key.At(next), next).is_none(), "an invalidated fact read as current");
}

#[test]
fn Test_An_Unrelated_Fact_Should_Be_Retained()
{
    let key = Base();
    let mut elsewhere = Base();
    elsewhere.subject = Subject(9);
    let mut store = Stored(&key);
    let fact = Fact(&elsewhere, GenerationId::INITIAL);
    store.Materialize(fact, &[]).expect("materializes");
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, key.subject, next);

    assert_eq!(report.Invalidated(), 1, "invalidation flushed the store");
    assert!(
        store.Current(&elsewhere.At(next), next).is_some(),
        "a fact about another subject was invalidated"
    );
    assert_eq!(report.retained, 1);
}

#[test]
fn Test_Invalidation_Should_Follow_Dependency_Edges()
{
    let read = Base();
    let mut derived = Base();
    derived.contract = CapabilityId::New(SEMANTIC);
    derived.subject = Subject(7);
    let mut store = Stored(&read);
    Materialize_Reading(&mut store, &derived, &read, ReadOutcome::Materialized);
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, read.subject, next);

    assert_eq!(report.direct, vec![read]);
    assert_eq!(
        report.dependent,
        vec![derived.clone()],
        "the consumer of an invalidated fact stayed current"
    );
    assert!(store.Current(&derived.At(next), next).is_none());
}

#[test]
fn Test_Invalidation_Should_Follow_Edges_Transitively()
{
    let read = Base();
    let mut middle = Base();
    middle.subject = Subject(7);
    let mut outer = Base();
    outer.subject = Subject(8);

    let mut store = Stored(&read);
    for (key, upstream) in [(&middle, &read), (&outer, &middle)]
    {
        Materialize_Reading(&mut store, key, upstream, ReadOutcome::Materialized);
    }
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, read.subject, next);
    let mut expected = vec![middle, outer];
    expected.sort();

    assert_eq!(report.dependent, expected, "invalidation stopped one edge short");
}

#[test]
fn Test_An_Edge_Recorded_By_A_Missed_Read_Should_Still_Carry_Invalidation()
{
    let absent = Base();
    let mut consumer = Base();
    consumer.subject = Subject(7);

    let mut store = MemoryFactStore::New();
    let appearing = Fact(&absent, GenerationId::INITIAL);
    Materialize_Reading(&mut store, &consumer, &absent, ReadOutcome::Absent);
    store.Materialize(appearing, &[]).expect("materializes");
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, absent.subject, next);

    assert_eq!(
        report.dependent,
        vec![consumer],
        "a judgement made because a fact was missing survived that fact arriving"
    );
}

#[test]
fn Test_An_Invalidated_Fact_Should_Stay_Readable_As_History()
{
    let key = Base();
    let mut store = Stored(&key);
    let next = GenerationId::INITIAL.Next();
    store.Invalidate(
        &GenerationCause::ConfigurationChanged {
            configuration: key.configuration,
        },
        next,
    );

    let (fact, supersession) = store.Historical(&key).expect("history");

    assert_eq!(fact.Generation(), GenerationId::INITIAL);
    assert_eq!(supersession.invalidated_at, next);
    assert!(supersession.cause.contains("configuration"), "{}", supersession.cause);
}

#[test]
fn Test_A_Live_Fact_Should_Have_No_History()
{
    assert!(
        Stored(&Base()).Historical(&Base()).is_none(),
        "a current fact was reported as superseded"
    );
}

#[test]
fn Test_A_Coarse_Provider_Should_Have_Its_Broadening_Reported()
{
    let key = Base();
    let mut store = MemoryFactStore::New();
    let mut fact = Fact(&key, GenerationId::INITIAL);
    fact.guarantee = Coarse();
    store.Materialize(fact, &[]).expect("materializes");
    let next = GenerationId::INITIAL.Next();

    let report = store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: key.subject,
            granularity: IncrementalGranularity::Symbol,
        },
        next,
    );

    assert_eq!(report.broadened.len(), 1, "{report:?}");
    assert_eq!(
        report.broadened.first().map(|broadening| broadening.applied),
        Some(IncrementalGranularity::WholeWorkspace),
        "a symbol-level cause was reported as if the provider could refresh a symbol"
    );
}

#[test]
fn Test_A_Provider_At_The_Requested_Granularity_Should_Not_Report_Broadening()
{
    let key = Base();
    let mut store = Stored(&key);

    let report = store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: key.subject,
            granularity: IncrementalGranularity::File,
        },
        GenerationId::INITIAL.Next(),
    );

    assert!(report.broadened.is_empty(), "{report:?}");
}

#[test]
fn Test_Every_Read_Should_Record_An_Edge()
{
    let key = Base();
    let store = Stored(&key);
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    assert!(reader.Get(&key.clone().At(GenerationId::INITIAL)).is_ok());
    assert!(reader.Get(&Varied(Component::Subject).At(GenerationId::INITIAL)).is_err());
    let _ = reader.Require(
        &CapabilityId::New(SYNTAX),
        &Subject(1),
        InputDigest::Of(&[b"fn main() {}"]),
        &Needing(Syntactic()),
    );

    assert_eq!(reader.Dependencies().len(), 3, "a read left no edge behind");
}

#[test]
fn Test_A_Read_That_Misses_Should_Record_Its_Miss()
{
    let store = MemoryFactStore::New();
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    assert!(reader.Get(&Base().At(GenerationId::INITIAL)).is_err());

    assert_eq!(
        reader.Dependencies().first().map(|dependency| dependency.outcome),
        Some(ReadOutcome::Absent),
        "a miss recorded nothing, so a judgement made without a fact cannot be revisited \
         when the fact arrives"
    );
}

#[test]
fn Test_A_Superseded_Read_Should_Record_Its_Supersession()
{
    let key = Base();
    let mut store = Stored(&key);
    let next = GenerationId::INITIAL.Next();
    store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: key.subject,
            granularity: IncrementalGranularity::File,
        },
        next,
    );
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(next));

    let refusal = reader.Get(&key.At(next)).expect_err("must refuse");

    assert!(format!("{refusal}").contains("invalidated"), "{refusal}");
    assert_eq!(
        reader.Dependencies().first().map(|dependency| dependency.outcome),
        Some(ReadOutcome::Superseded)
    );
}

#[test]
fn Test_The_Recorded_Edges_Should_Become_The_Stored_Dependencies()
{
    let read = Base();
    let mut derived = Base();
    derived.subject = Subject(7);
    let mut store = Stored(&read);
    let registry = Offering(Syntactic());

    let dependencies = {
        let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));
        let _ = reader.Get(&read.clone().At(GenerationId::INITIAL));
        reader.Into_Dependencies()
    };
    let fact = Fact(&derived, GenerationId::INITIAL);
    store.Materialize(fact, &dependencies).expect("materializes");

    assert_eq!(store.Dependencies_Of(&derived).len(), 1);
    assert_eq!(
        store.Dependencies_Of(&derived).first().map(|dependency| dependency.key.clone()),
        Some(read)
    );
}

#[test]
fn Test_Require_Should_Answer_When_The_Registry_Resolves_And_The_Fact_Exists()
{
    let key = Base();
    let store = Stored(&key);
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let fact = reader.Require(
        &CapabilityId::New(SYNTAX),
        &Subject(1),
        InputDigest::Of(&[b"fn main() {}"]),
        &Needing(Syntactic()),
    );

    assert!(fact.is_ok(), "the honest path does not answer, so the refusals prove nothing");
}

#[test]
fn Test_Require_Should_Return_MissingCapability_When_Nothing_Declares_It()
{
    let store = Stored(&Base());
    let registry = Registry::New();
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let refusal = reader
        .Require(
            &CapabilityId::New(SYNTAX),
            &Subject(1),
            InputDigest::Of(&[b"fn main() {}"]),
            &Needing(Syntactic()),
        )
        .expect_err("must refuse");

    assert_eq!(refusal, Applicability::MissingCapability);
    assert_ne!(
        refusal,
        Applicability::NotApplicable,
        "coverage debt was reported as a judgement about the subject"
    );
}

#[test]
fn Test_Require_Should_Refuse_A_Provider_Below_The_Requirement()
{
    let store = Stored(&Base());
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let semantic = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
    let refusal = reader
        .Require(
            &CapabilityId::New(SYNTAX),
            &Subject(1),
            InputDigest::Of(&[b"fn main() {}"]),
            &Needing(semantic),
        )
        .expect_err("must refuse");
    let answered = reader.Dependencies().iter().any(|edge| return edge.outcome.Answered());

    assert_eq!(refusal, Applicability::MissingCapability);
    assert!(!answered, "a fact was returned for a requirement no provider reaches");
}

#[test]
fn Test_Require_Should_Not_Fabricate_A_Fact_The_Store_Does_Not_Hold()
{
    let store = MemoryFactStore::New();
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let refusal = reader
        .Require(
            &CapabilityId::New(SYNTAX),
            &Subject(1),
            InputDigest::Of(&[b"fn main() {}"]),
            &Needing(Syntactic()),
        )
        .expect_err("must refuse");

    assert_eq!(refusal, Applicability::DependencyUnavailable);
    assert_eq!(
        reader.Dependencies().first().map(|dependency| dependency.outcome),
        Some(ReadOutcome::Degraded(Applicability::DependencyUnavailable))
    );
}

#[test]
fn Test_Require_Should_Refuse_A_Fact_Whose_Inputs_Differ()
{
    let store = Stored(&Base());
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let refusal = reader
        .Require(
            &CapabilityId::New(SYNTAX),
            &Subject(1),
            InputDigest::Of(&[b"fn main() { edited }"]),
            &Needing(Syntactic()),
        )
        .expect_err("must refuse");

    assert_eq!(
        refusal,
        Applicability::DependencyUnavailable,
        "a fact about other inputs was served as this subject's analysis"
    );
}

#[test]
fn Test_The_Store_Trait_Should_Be_Sealed()
{
    let key = Base();
    let store = Stored(&key);
    let sealed: &dyn FactStore = &store;

    assert!(
        sealed
            .Current(&key.At(GenerationId::INITIAL), GenerationId::INITIAL)
            .is_some(),
        "the trait is reachable as an object and answers nothing"
    );
}
