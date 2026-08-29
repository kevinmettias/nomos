//! Computing one module index from the member facts it depends on.

use super::{FactReader, MemoryFactStore, Against, Module, Rolled, FactError, FactKey, ModuleIndex, FactContext, MaterializedFact, EvidenceClass, Declared_Guarantee, FactPayload, Payload_Schema, Encode_Index, SubjectId, ModuleMember, Capability, CONTRACT_VERSION, ProviderId, PROVIDER, GuaranteeDigest, InputDigest, Digest128, Dependency, Reader, Context, Requirement, MemberReading, Outcome, Applicability, IndexEntry};

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
pub(super) fn Rollup_Fact(key: &FactKey, index: &ModuleIndex, context: FactContext) -> MaterializedFact
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
pub fn Index_Key(module: SubjectId, members: &[ModuleMember], context: FactContext) -> FactKey
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
pub(super) fn Canonical_Members(members: &[ModuleMember]) -> Vec<ModuleMember>
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
pub(super) fn Index_Inputs(members: &[ModuleMember]) -> InputDigest
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
    pub(super) index: ModuleIndex,
    pub(super) dependencies: Vec<Dependency>,
}

/// Reads every member's syntax fact through the registry and indexes what they declare.
pub(super) fn Read_Members(
    store: &MemoryFactStore,
    against: &Against<'_>,
    module: SubjectId,
    members: &[ModuleMember],
) -> Members
{
    let context = Reading_Context(against.context);
    let mut reader = Reader::On(store, against.registry, context);
    let mut index = ModuleIndex {
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
    index: &mut ModuleIndex,
    reader: &mut Reader<'_, '_>,
    member: &ModuleMember,
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
    member: &ModuleMember,
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
