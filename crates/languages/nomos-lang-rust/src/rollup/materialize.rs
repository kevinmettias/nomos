//! Computing one module index from the member facts it depends on.

use super::{FactReader, MemoryFactStore, Against, Module, Rolled, FactError, FactKey, Index, FactContext, MaterializedFact, EvidenceClass, Declared_Guarantee, FactPayload, Payload_Schema, Encode_Index, SubjectId, Member, Capability, CONTRACT_VERSION, ProviderId, PROVIDER, GuaranteeDigest, InputDigest, Digest128, Dependency, Reader, Context, Requirement, MemberReading, Outcome, Applicability, IndexEntry};

/// Rolls a module's members up into a derived fact and writes it, with its edges.
///
/// `need` is the caller's requirement for `nomos-cap-syntax`'s capability, passed in rather
/// than written here. A rollup that composed its own would resolve a provider the run did
/// not choose, rebuild a key nobody wrote, and report every member unreachable — loudly,
/// and for the wrong reason.
///
/// Members are put in a canonical order and deduplicated by subject before anything is
/// read, so the same module described in two orders is one fact rather than two. A module
/// is a set of files; naming one twice does not make it two members.
///
/// # Errors
///
/// Whatever the store refuses the write for — in practice
/// [`FactError::Backdated`], which means the caller is materializing into a generation the
/// store has already left.
pub fn Materialize_Index(
    store: &mut MemoryFactStore,
    against: &Against<'_>,
    module: &Module,
) -> Result<Rolled, FactError>
{
    let members = Canonical_Members(&module.members);
    let key = Index_Key(module.subject, &members, against.context);

    // The read borrows the store immutably and the write needs it mutably, so the reader
    // is confined to this scope. What survives it is owned: the index, and the edges the
    // reader observed on the way to it.
    let Members {
        index,
        dependencies,
    } = Read_Members(store, against, module.subject, &members);

    let fact = Rollup_Fact(&key, &index, against.context);
    store.Materialize(fact, &dependencies)?;

    return Ok(Rolled {
        key,
        index,
        dependencies,
    });
}

/// The rollup as a fact.
///
/// The snapshot is provenance — the tree this rollup was computed over — and not part of
/// the key, so the workspace moving re-addresses nothing, per `OD-ANALYSIS-001`.
///
/// The evidence is no stronger than what it derived from. Promoting it to `Verified` would
/// launder the rollup's own arithmetic into a measurement.
pub(super) fn Rollup_Fact(key: &FactKey, index: &Index, context: FactContext) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Derived,
        guarantee: Declared_Guarantee(),
        payload: FactPayload::New(Payload_Schema(), Encode_Index(index)),
    };
}

/// The key a module's index is filed under.
///
/// Exposed because a caller that wants to know whether a rollup is already held, or to read
/// one back, needs the key without recomputing it.
#[must_use]
pub fn Index_Key(module: SubjectId, members: &[Member], context: FactContext) -> FactKey
{
    return FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject: module,
        semantic_inputs: Index_Inputs(&Canonical_Members(members)),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&Declared_Guarantee()),
        variant: context.variant,
        configuration: context.configuration,
    };
}

/// The members in the one order this provider reads them in.
pub(super) fn Canonical_Members(members: &[Member]) -> Vec<Member>
{
    let mut ordered = members.to_vec();
    ordered.sort_by_key(|member| {
        return (*member.subject.Digest().Bytes(), *member.inputs.Digest().Bytes());
    });
    ordered.dedup_by_key(|member| return *member.subject.Digest().Bytes());

    return ordered;
}

/// What the rollup was computed from: every member's subject and every member's inputs.
///
/// Both halves are load-bearing. Without the inputs, editing a member leaves the rollup's
/// key unchanged and a stale answer stays addressable as a current one. Without the
/// subjects, a module that swapped one file for another holding identical bytes would key
/// the same, and a module is which files it has and not only what they contain.
pub(super) fn Index_Inputs(members: &[Member]) -> InputDigest
{
    let mut parts: Vec<[u8; Digest128::BYTE_LENGTH]> = Vec::new();
    for member in members
    {
        parts.push(*member.subject.Digest().Bytes());
        parts.push(*member.inputs.Digest().Bytes());
    }

    let borrowed: Vec<&[u8]> = parts.iter().map(|part| return part.as_slice()).collect();

    return InputDigest::Of(&borrowed);
}

/// What a reading of a module's members produced.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
pub(super) struct Members
{
    pub(super) index: Index,
    pub(super) dependencies: Vec<Dependency>,
}

/// Reads every member's syntax fact through the registry and indexes what they declare.
pub(super) fn Read_Members(
    store: &MemoryFactStore,
    against: &Against<'_>,
    module: SubjectId,
    members: &[Member],
) -> Members
{
    let context = Reading_Context(against.context);
    let mut reader = Reader::On(store, against.registry, context);
    let mut index = Index {
        module,
        members: Vec::new(),
        items: Vec::new(),
    };

    for member in members
    {
        Index_Member(&mut index, &mut reader, member, against.need);
    }

    return Members {
        index,
        dependencies: reader.Into_Dependencies(),
    };
}

/// The reading context, as the reader takes it.
pub(super) fn Reading_Context(context: FactContext) -> Context
{
    return Context {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// One member's declarations, or the fact that it could not be read.
///
/// `Require_Any` rather than `Require`: reading only the chosen provider leaves a weaker
/// provider's answer written into the store and unread, and the rollup would report the
/// member missing while the answer sat one lookup away.
///
/// Bytes filed under the syntax schema that are not the syntax schema count as unreachable
/// rather than as an empty answer — a member that could not be read must not encode like a
/// member that declares nothing.
pub(super) fn Index_Member(
    index: &mut Index,
    reader: &mut Reader<'_, '_>,
    member: &Member,
    need: &Requirement,
)
{
    let Some((payload, applicability)) = Declared_By(reader, member, need)
    else
    {
        index.members.push(MemberReading {
            subject: member.subject,
            outcome: Outcome::Unreachable,
        });

        return;
    };

    index.members.push(MemberReading {
        subject: member.subject,
        outcome: Outcome_Of(applicability),
    });
    for item in payload.items
    {
        let entry = Entry_Of(member.subject, item);
        index.items.push(entry);
    }
}

/// What one member's fact says, and how good the answer was.
pub(super) fn Declared_By(
    reader: &mut Reader<'_, '_>,
    member: &Member,
    need: &Requirement,
) -> Option<(nomos_cap_syntax::SyntaxPayload, Applicability)>
{
    let capability = nomos_cap_syntax::Capability();
    let (fact, applicability) = reader
        .Require_Any(&capability, &member.subject, member.inputs, need)
        .ok()?;
    let payload = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes).ok()?;

    return Some((payload, applicability));
}

/// One declared item, filed under the member that declared it.
pub(super) fn Entry_Of(member: SubjectId, item: nomos_cap_syntax::PayloadItem) -> IndexEntry
{
    return IndexEntry {
        member,
        ordinal: item.ordinal,
        kind: item.kind,
        visibility: item.visibility,
        qualified_name: item.qualified_name,
    };
}

/// Whether the answer came from the chosen provider or from a weaker one below it.
pub(super) fn Outcome_Of(applicability: Applicability) -> Outcome
{
    if applicability == Applicability::SupportedWithFallback
    {
        return Outcome::Approximate;
    }

    return Outcome::Read;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::Registry;
    use nomos_contracts::{Assurance, BuildVariantId, ConfigurationId, FactVariant, GenerationId, Guarantee, IncrementalGranularity, SnapshotId};

    #[test]
    fn Test_Index_Key_Should_Not_Depend_On_The_Members_Own_Order()
    {
        let a = Member::Of(Subject("a.rs"), "pub fn A() {}\n");
        let b = Member::Of(Subject("b.rs"), "pub fn B() {}\n");
        let context = Test_Context();

        let forwards = Index_Key(Subject("the/module"), &[a, b], context);
        let backwards = Index_Key(Subject("the/module"), &[b, a], context);

        assert_eq!(forwards.Digest(), backwards.Digest());
    }

    #[test]
    fn Test_Canonical_Members_Should_Sort_And_Deduplicate_By_Subject()
    {
        let a = Member::Of(Subject("a.rs"), "pub fn A() {}\n");
        let b = Member::Of(Subject("b.rs"), "pub fn B() {}\n");
        let a_again = Member::Of(Subject("a.rs"), "pub fn A() {}\n");

        let ordered = Canonical_Members(&[b, a, a_again]);

        assert_eq!(ordered.len(), 2, "the repeated subject must collapse to one member");
        assert_eq!(ordered.first().expect("two members").subject, a.subject);
    }

    #[test]
    fn Test_Index_Inputs_Should_Depend_On_Both_Subject_And_Content()
    {
        let a = Member::Of(Subject("a.rs"), "pub fn A() {}\n");
        let a_edited = Member::Of(Subject("a.rs"), "pub fn A() {}\npub fn B() {}\n");
        let renamed = Member::Of(Subject("b.rs"), "pub fn A() {}\n");

        let original = Index_Inputs(&[a]);

        assert_ne!(original, Index_Inputs(&[a_edited]), "editing a member must change the inputs");
        assert_ne!(original, Index_Inputs(&[renamed]), "renaming a member must change the inputs");
    }

    #[test]
    fn Test_Reading_Context_Should_Carry_Every_Field_The_Fact_Context_Has()
    {
        let context = Test_Context();
        let reading = Reading_Context(context);

        assert_eq!(reading.snapshot, context.snapshot);
        assert_eq!(reading.variant, context.variant);
        assert_eq!(reading.configuration, context.configuration);
        assert_eq!(reading.generation, context.generation);
    }

    #[test]
    fn Test_Entry_Of_Should_File_The_Item_Under_Its_Declaring_Member()
    {
        let member = Subject("alpha.rs");
        let item = nomos_cap_syntax::PayloadItem {
            ordinal: 3,
            kind: "Function".to_owned(),
            visibility: "Public".to_owned(),
            qualified_name: "Alpha".to_owned(),
            documentation: nomos_cap_syntax::Observation::Absent,
            shape: nomos_cap_syntax::Observation::Absent,
        };

        let entry = Entry_Of(member, item);

        assert_eq!(entry.member, member);
        assert_eq!(entry.ordinal, 3);
        assert_eq!(entry.qualified_name, "Alpha");
    }

    #[test]
    fn Test_Outcome_Of_Should_Be_Approximate_Only_For_A_Fallback_Applicability()
    {
        assert_eq!(Outcome_Of(Applicability::SupportedWithFallback), Outcome::Approximate);
        assert_eq!(Outcome_Of(Applicability::Supported), Outcome::Read);
    }

    #[test]
    fn Test_Declared_By_Should_Be_None_When_The_Capability_Was_Never_Declared()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut reader = Reader::On(&store, &registry, Reading_Context(Test_Context()));
        let member = Member::Of(Subject("alpha.rs"), "pub fn Alpha() {}\n");

        assert!(Declared_By(&mut reader, &member, &Need()).is_none());
    }

    #[test]
    fn Test_Index_Member_Should_Record_Unreachable_When_Nothing_Was_Declared()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut reader = Reader::On(&store, &registry, Reading_Context(Test_Context()));
        let mut index = Index {
            module: Subject("the/module"),
            members: Vec::new(),
            items: Vec::new(),
        };
        let member = Member::Of(Subject("alpha.rs"), "pub fn Alpha() {}\n");

        Index_Member(&mut index, &mut reader, &member, &Need());

        let recorded = index.members.first().expect("one member recorded");
        assert_eq!(recorded.outcome, Outcome::Unreachable);
        assert!(index.items.is_empty());
    }

    #[test]
    fn Test_Read_Members_Should_Report_Every_Member_Unreachable_With_No_Registered_Provider()
    {
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let need = Need();
        let against = Against {
            registry: &registry,
            need: &need,
            context: Test_Context(),
        };
        let member = Member::Of(Subject("alpha.rs"), "pub fn Alpha() {}\n");

        let Members { index, dependencies: _ } = Read_Members(&store, &against, Subject("the/module"), &[member]);

        assert_eq!(index.Unreachable(), 1, "a capability nothing declared has no readable answer for any member");
    }

    #[test]
    fn Test_Rollup_Fact_Should_Carry_The_Declared_Guarantee_And_Derived_Evidence()
    {
        let key = Index_Key(Subject("the/module"), &[], Test_Context());
        let index = Index {
            module: Subject("the/module"),
            members: Vec::new(),
            items: Vec::new(),
        };

        let fact = Rollup_Fact(&key, &index, Test_Context());

        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.evidence, EvidenceClass::Derived);
    }

    #[test]
    fn Test_Materialize_Index_Should_Write_A_Rollup_Fact_For_An_Unreachable_Module()
    {
        let mut store = MemoryFactStore::New();
        let registry = Registry::New();
        let need = Need();
        let against = Against {
            registry: &registry,
            need: &need,
            context: Test_Context(),
        };
        let module = Module {
            subject: Subject("the/module"),
            members: vec![Member::Of(Subject("alpha.rs"), "pub fn Alpha() {}\n")],
        };

        let rolled = Materialize_Index(&mut store, &against, &module).expect("materializes even with nothing readable");

        assert_eq!(rolled.index.Unreachable(), 1);
    }

    fn Subject(path: &str) -> SubjectId
    {
        use nomos_model::Content_Digest;

        return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
    }

    fn Test_Context() -> FactContext
    {
        return FactContext {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; Digest128::BYTE_LENGTH])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; Digest128::BYTE_LENGTH])),
            generation: GenerationId::INITIAL,
        };
    }

    /// The syntax capability's requirement every reader below asks against — an empty
    /// registry cannot satisfy it, which is exactly the "nothing was declared" case these
    /// tests need.
    fn Need() -> Requirement
    {
        let guarantee = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );

        return Requirement::New(nomos_cap_syntax::Capability(), nomos_cap_syntax::CONTRACT_VERSION, guarantee);
    }
}
