//! A fact computed from other facts, and the dependency edges that come with it.
//!
//! # What was missing
//!
//! `nomos-analysis` ships a complete transitive invalidation mechanism — a reverse index of
//! dependents, a [`nomos_analysis::Reader`] that records every read, and an
//! [`nomos_analysis::InvalidationReport`] that separates what a change hit directly from
//! what it reached through an edge. Nothing that ships created an edge. Every fact this
//! workspace materializes outside a test is a leaf, so
//! [`nomos_analysis::InvalidationReport::dependent`] was a field no run could make
//! non-empty.
//!
//! The reason was structural rather than an omission. Every fact is filed under a
//! capability, and the one this crate answered — `nomos-cap-syntax`'s — is a per-file leaf
//! by construction: its semantic input is one file's text and nothing else. A capability
//! whose answer is a function of one file can never depend on another answer. So the
//! dependent half of invalidation needed a *second* capability before it could have a
//! producer, and this module is it. `docs/records/OD-ANALYSIS-002` records the reasoning.
//!
//! # The question this capability asks
//!
//! *Which items does a module declare, and which of its files declared each one.*
//!
//! That is deliberately not the syntax capability's question asked over more subjects. A
//! union of syntax payloads would lose the only thing that makes a module-level answer
//! worth having: [`nomos_cap_syntax::PayloadItem`] carries a name qualified by syntactic
//! nesting *within one file* and has no field for which file that was, so two files each
//! declaring `Read` produce two indistinguishable records. Every entry here names its
//! member, which is the field the rollup exists to add.
//!
//! # Why the contract lives beside its provider
//!
//! `OD-CAPABILITY-002` sets the criterion and it is contention, not principle: a capability
//! with a single provider is not wrongly filed for living beside that provider, and moving
//! it out would buy a crate and no property. `nomos-cap-syntax` exists because two crates
//! offer against it and neither may name the other. Nothing offers against this one but the
//! function below. The day something else does, this belongs under `crates/capabilities`
//! and `Test_A_Capability_Id_Should_Be_Written_In_One_Crate` will say so the moment the id
//! is spelled twice.
//!
//! # Why the granularity is Project
//!
//! A rollup over a module cannot refresh half a module. Declaring
//! [`IncrementalGranularity::File`] would be claiming a precision this has no way to
//! deliver, and the invalidation engine would refresh one member's contribution and treat
//! the rest as current. [`IncrementalGranularity::Project`] makes the engine broaden a
//! file-granular cause and record on
//! [`nomos_analysis::InvalidationReport::broadened`] that it did — the cost of the rollup,
//! stated rather than absorbed.
//!
//! # Why the edges are not a list written here
//!
//! [`Materialize_Index`] never assembles a dependency array. It reads through a
//! [`nomos_analysis::Reader`] and hands the store what the reader observed. A hand-written
//! edge list is a *claim* about what was read; this is a record of it, and the two diverge
//! the first time a read is added and the list is not. That includes the reads that found
//! nothing: a member with no fact is still an edge, because the day the parser does have
//! something for it, this rollup is stale and only the edge knows.
//!
//! # Determinism
//!
//! Same triple as [`crate::SyntaxFactProduction`], which is this crate's declaration and
//! covers it: the member order is canonical rather than the caller's, the payload is
//! hand-encoded in one place, and nothing in the path touches a clock, a path separator or
//! an unordered collection. A second `impl Strategy` naming the same row is deliberately
//! not added — `tests/contract`'s determinism guard requires every declaration to be held
//! to it by the integration harness, so a declaration added without one there is a promise
//! nothing can falsify, which is the defect that guard exists to catch.

mod against;
mod index_entry;
mod member_reading;
mod module;
mod module_index;
mod module_member;
mod outcome;
mod rolled;

pub use against::Against;
pub use index_entry::IndexEntry;
pub use member_reading::MemberReading;
pub use module::Module;
pub use module_index::ModuleIndex;
pub use module_member::ModuleMember;
pub use outcome::Outcome;
pub use rolled::Rolled;

use crate::provider::FactContext;
use nomos_analysis::{
    Context, Dependency, FactError, FactKey, FactPayload, FactReader, GuaranteeDigest, InputDigest,
    MaterializedFact, MemoryFactStore, Reader,
};
use nomos_capability::{CapabilityContract, ProviderOffer, Requirement};
use nomos_contracts::{
    Applicability, Assurance, CapabilityId, ContractVersion, Digest128, EvidenceClass, FactVariant,
    Guarantee, IncrementalGranularity, ProviderId, SchemaId, SubjectId,
};

/// The capability this module answers.
///
/// Named for what a caller gets rather than for how it is obtained, by the same rule that
/// named `nomos.cap.syntax.items`: a second provider — one that read a module from a
/// compiler's own item table instead of from syntax facts — must be able to offer this
/// honestly.
pub const CAPABILITY: &str = "nomos.cap.module.index";

/// This implementation. The method is in the name because the method is a fact about the
/// answer: this one is a rollup over stored facts and is exactly as good as they were.
pub const PROVIDER: &str = "nomos.lang.rust.rollup";

/// The payload schema every answer to this capability is stamped with.
///
/// Versioned separately from the contract because the shape of the bytes and the meaning
/// of the question change for different reasons.
pub const SCHEMA: &str = "nomos.module.index.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The `outcome` field for a member read from the provider the caller asked for.
pub const READ: &str = "read";

/// The `outcome` field for a member answered by a weaker provider than the caller asked
/// for.
pub const APPROXIMATE: &str = "approximate";

/// The `outcome` field for a member with no readable answer.
pub const UNREACHABLE: &str = "unreachable";

/// Fields in a `module` record, counting the tag.
const MODULE_FIELDS: usize = 2;

/// Fields in a `member` record, counting the tag.
const MEMBER_FIELDS: usize = 3;

/// Fields in an `item` record, counting the tag.
const ITEM_FIELDS: usize = 6;

#[must_use]
pub fn Capability() -> CapabilityId
{
    return CapabilityId::New(CAPABILITY);
}

#[must_use]
pub fn Payload_Schema() -> SchemaId
{
    return SchemaId::New(SCHEMA);
}

/// The strongest anything may claim for this capability.
///
/// Deliberately above what the provider below achieves. A rollup that reparsed only the
/// member that changed would be file-granular and complete, and a ceiling set to today's
/// implementation would have to be raised to admit it — a ceiling that moves is not a
/// ceiling.
///
/// [`FactVariant::Syntactic`] is the ceiling because an index of what files declare is a
/// statement about what they say on their face. A provider that resolved the names it
/// indexed would be answering a different question.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
}

/// The contract, to be declared once by whichever composition root builds a registry.
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: CONTRACT_VERSION,
        summary: "The items a module declares, each attributed to the member file that \
                  declared it, together with which members could be read and which were \
                  answered only approximately."
            .to_owned(),
        ceiling: Ceiling(),
    };
}

/// What this provider claims, on every axis.
///
/// Every axis is at most what its inputs were, which is what
/// [`EvidenceClass::Derived`] says about provenance stated as a guarantee.
///
/// [`Assurance::Sound`] carries over: an entry is here because a member's fact contained
/// it, and there is no step by which this could report an item no member declared.
/// Completeness stays [`Assurance::Unknown`] and cannot be anything else — the syntax facts
/// this reads may have missed macro-generated items, so this has missed them too, and a
/// rollup that is complete over incomplete inputs would be claiming to have seen what its
/// own sources could not.
///
/// [`IncrementalGranularity::Project`] is the module doc's reason: there is no partial
/// refresh to offer.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

/// This provider's offer against [`Capability_Contract`].
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: Capability(),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

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
    let members = Canonical(&module.members);
    let key = Index_Key(module.subject, &members, against.context);

    // The read borrows the store immutably and the write needs it mutably, so the reader
    // is confined to this scope. What survives it is owned: the index, and the edges the
    // reader observed on the way to it.
    let (index, dependencies) = Read_Members(store, against, module.subject, &members);

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
fn Rollup_Fact(key: &FactKey, index: &ModuleIndex, context: FactContext) -> MaterializedFact
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
        semantic_inputs: Index_Inputs(&Canonical(members)),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&Declared_Guarantee()),
        variant: context.variant,
        configuration: context.configuration,
    };
}

/// The members in the one order this provider reads them in.
fn Canonical(members: &[ModuleMember]) -> Vec<ModuleMember>
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
fn Index_Inputs(members: &[ModuleMember]) -> InputDigest
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

/// Reads every member's syntax fact through the registry and indexes what they declare.
fn Read_Members(
    store: &MemoryFactStore,
    against: &Against<'_>,
    module: SubjectId,
    members: &[ModuleMember],
) -> (ModuleIndex, Vec<Dependency>)
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

    return (index, reader.Into_Dependencies());
}

/// The reading context, as the reader takes it.
fn Reading_Context(context: FactContext) -> Context
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
fn Index_Member(
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
fn Declared_By(
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
fn Entry_Of(member: SubjectId, item: nomos_cap_syntax::PayloadItem) -> IndexEntry
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
fn Outcome_Of(applicability: Applicability) -> Outcome
{
    if applicability == Applicability::SupportedWithFallback
    {
        return Outcome::Approximate;
    }

    return Outcome::Read;
}

/// The canonical byte encoding of an index.
///
/// Line-oriented text by the same rules and for the same reasons as the syntax schema's:
/// tab-separated fields, `\n` terminators and never `\r\n`, written by hand in one place so
/// no dependency's field ordering can silently re-address every rollup in the store.
///
/// ```text
/// payload := module member* item*
/// module  := "module" TAB subject LF
/// member  := "member" TAB subject TAB outcome LF
/// item    := "item" TAB subject TAB ordinal TAB kind TAB visibility TAB qualified-name LF
/// outcome := "read" | "approximate" | "unreachable"
/// subject := 32 lowercase hexadecimal characters
/// ```
///
/// The `module` record is first and appears exactly once. It is what makes an index over a
/// module with no members a payload rather than the empty byte string, which the reader
/// refuses — a module nothing was read for and a module that declares nothing are two
/// answers and must not share an encoding.
///
/// **A record is exactly the fields the grammar gives it**, and one carrying more is refused
/// rather than read down to the fields this build knows. That is the same rule as the one
/// for an unrecognised record tag and it is refused for the same reason: a longer record is
/// most likely a newer schema, so the field being dropped is the one carrying what changed.
/// [`Expect_Fields`] is where it is enforced.
///
/// Nothing is escaped. `kind`, `visibility` and `qualified-name` arrive from the syntax
/// schema's unescaped fields, which are tab-free and newline-free by that schema's own
/// grammar, so an escape here would be a second encoding of bytes that already passed
/// through one.
#[must_use]
pub fn Encode_Index(index: &ModuleIndex) -> Vec<u8>
{
    let mut encoded = String::new();

    encoded.push_str("module\t");
    encoded.push_str(&index.module.Digest().to_string());
    encoded.push('\n');

    for member in &index.members
    {
        encoded.push_str("member\t");
        encoded.push_str(&member.subject.Digest().to_string());
        encoded.push('\t');
        encoded.push_str(member.outcome.Label());
        encoded.push('\n');
    }

    for item in &index.items
    {
        Encode_Entry(&mut encoded, item);
    }

    return encoded.into_bytes();
}

/// One item record: the member that declared it, then the syntax schema's own fields.
fn Encode_Entry(encoded: &mut String, item: &IndexEntry)
{
    encoded.push_str("item\t");
    encoded.push_str(&item.member.Digest().to_string());
    encoded.push('\t');
    encoded.push_str(&item.ordinal.to_string());
    encoded.push('\t');
    encoded.push_str(&item.kind);
    encoded.push('\t');
    encoded.push_str(&item.visibility);
    encoded.push('\t');
    encoded.push_str(&item.qualified_name);
    encoded.push('\n');
}

/// Reads an index payload back.
///
/// The reader for this schema is here, and it is the only one. A consumer writing its own
/// is not proving anything — it is re-deciding what a well-formed payload is, which is the
/// defect `P10-SYNTAX-SCHEMA` found in the schema next door with three answers to that
/// question.
///
/// # Errors
///
/// Returns what was refused and where. A decoder that fell back to an empty index would
/// report every unreadable rollup as a module that declares nothing, which is
/// indistinguishable from a module that genuinely does.
pub fn Parse_Index(payload: &[u8]) -> Result<ModuleIndex, String>
{
    let text = core::str::from_utf8(payload).map_err(|error| {
        return format!("the payload is not UTF-8, so it is not this schema: {error}");
    })?;

    let mut lines = text.lines().enumerate();
    let mut index = Opened(&mut lines)?;

    for (offset, line) in lines
    {
        Read_Record(&mut index, line, offset.saturating_add(1))?;
    }

    return Ok(index);
}

/// The `module` record, and the empty index it opens.
///
/// It appears exactly once and first. That is what makes an index over a module with no
/// members a payload rather than the empty byte string — a module nothing was read for and
/// a module that declares nothing are two answers and must not share an encoding.
fn Opened(lines: &mut core::iter::Enumerate<core::str::Lines<'_>>) -> Result<ModuleIndex, String>
{
    let Some((_, header)) = lines.next()
    else
    {
        return Err("the payload is empty, which is not a module that declares nothing — \
                    that is a `module` record and no members"
            .to_owned());
    };

    let fields: Vec<&str> = header.split('\t').collect();
    if fields.first().copied() != Some("module")
    {
        return Err(format!("the first record is `{header}` and not a `module` record"));
    }
    Expect_Fields("module", &fields, MODULE_FIELDS, 1)?;

    return Ok(ModuleIndex {
        module: Subject_From(fields.get(1).copied().unwrap_or_default(), 1)?,
        members: Vec::new(),
        items: Vec::new(),
    });
}

/// One record after the header.
///
/// An unrecognised tag is refused rather than skipped, for the reason the schema gives: a
/// record this build does not know is most likely a newer schema, and passing over it reads
/// the payload down to the part that has not changed.
fn Read_Record(index: &mut ModuleIndex, line: &str, number: usize) -> Result<(), String>
{
    let fields: Vec<&str> = line.split('\t').collect();

    // `split` yields at least one element for every input, so `first` is only `None` for an
    // iterator that is already exhausted, which this one is not.
    match fields.first().copied().unwrap_or_default()
    {
        "member" =>
        {
            let member = Member_Record(&fields, number)?;
            index.members.push(member);
        }
        "item" =>
        {
            let item = Item_Record(&fields, number)?;
            index.items.push(item);
        }
        tag => return Err(Unreadable_Record(tag, number)),
    }

    return Ok(());
}

/// A record this build will not read.
///
/// A second `module` record is refused because the first one is the index's identity and a
/// payload carrying two does not say which. Anything else is refused because a tag this
/// build does not know is most likely a newer schema, and passing over it would read the
/// payload down to the part that has not changed.
fn Unreadable_Record(tag: &str, number: usize) -> String
{
    if tag == "module"
    {
        return format!("line {number} is a second `module` record");
    }

    return format!("line {number} has record tag `{tag}`, which this build does not understand");
}

fn Member_Record(fields: &[&str], line: usize) -> Result<MemberReading, String>
{
    Expect_Fields("member", fields, MEMBER_FIELDS, line)?;

    let outcome = match fields.get(2).copied().unwrap_or_default()
    {
        READ => Outcome::Read,
        APPROXIMATE => Outcome::Approximate,
        UNREACHABLE => Outcome::Unreachable,
        other => return Err(format!("`{other}` on line {line} is not an outcome")),
    };

    return Ok(MemberReading {
        subject: Subject_From(fields.get(1).copied().unwrap_or_default(), line)?,
        outcome,
    });
}

fn Item_Record(fields: &[&str], line: usize) -> Result<IndexEntry, String>
{
    Expect_Fields("item", fields, ITEM_FIELDS, line)?;

    let ordinal = fields.get(2).copied().unwrap_or_default();
    let ordinal: u32 = ordinal
        .parse()
        .map_err(|_| return format!("`{ordinal}` on line {line} is not an ordinal"))?;

    return Ok(IndexEntry {
        member: Subject_From(fields.get(1).copied().unwrap_or_default(), line)?,
        ordinal,
        kind: fields.get(3).copied().unwrap_or_default().to_owned(),
        visibility: fields.get(4).copied().unwrap_or_default().to_owned(),
        qualified_name: fields.get(5).copied().unwrap_or_default().to_owned(),
    });
}

/// Refuses a record whose field count is not the one the grammar states.
///
/// # Why a longer record is refused rather than truncated
///
/// The same argument the unknown record tag gets, one grain finer, and it took
/// `P10-INDEX-FIELD-STRICTNESS` to notice that this reader made it in one place and not the
/// other. A record carrying more fields than this build knows about is most likely a payload
/// from a newer schema, and that is exactly the case where reading the prefix and discarding
/// the rest is worst: the discarded field is where the new information is, and the caller is
/// handed a clean decode of a payload it only partly understood.
///
/// A shorter record is refused for the older reason — a missing field defaulting to empty is
/// an absence invented by the reader rather than one the writer wrote.
///
/// This is deliberately the same shape as `nomos-cap-syntax`'s `Expect_Fields`, which had it
/// right from the start. The two schemas share no code and should not: what they share is a
/// rule about what a record is, and agreeing by construction would remove the disagreement
/// that would otherwise be visible.
fn Expect_Fields(tag: &str, fields: &[&str], expected: usize, line: usize) -> Result<(), String>
{
    if fields.len() == expected
    {
        return Ok(());
    }

    return Err(format!(
        "the `{tag}` record on line {line} has {} field(s) and this schema's has {expected}. \
         A longer record is most likely a newer schema, and reading its first {expected} \
         fields would discard exactly the part that is new",
        fields.len()
    ));
}

/// A subject read back out of the hexadecimal the encoder wrote.
fn Subject_From(hexadecimal: &str, line: usize) -> Result<SubjectId, String>
{
    if hexadecimal.len() != Digest128::HEX_LENGTH
    {
        return Err(format!(
            "`{hexadecimal}` on line {line} is {} characters and a subject is {}",
            hexadecimal.len(),
            Digest128::HEX_LENGTH
        ));
    }

    let mut bytes = [0_u8; Digest128::BYTE_LENGTH];
    for (slot, pair) in bytes.iter_mut().zip(Pairs(hexadecimal))
    {
        *slot = u8::from_str_radix(pair, 16)
            .map_err(|_| return format!("`{pair}` on line {line} is not hexadecimal"))?;
    }

    return Ok(SubjectId::From_Digest(Digest128::From_Bytes(bytes)));
}

/// The two-character slices of an even-length ASCII hexadecimal string.
fn Pairs(hexadecimal: &str) -> impl Iterator<Item = &str>
{
    return (0..hexadecimal.len())
        .step_by(2)
        .filter_map(|start| return hexadecimal.get(start..start.saturating_add(2)));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::Registry;
    use nomos_capability::{Requirement, Resolution, Unmet};
    use nomos_model::Content_Digest;

    fn Subject(path: &str) -> SubjectId
    {
        return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
    }

    fn An_Index() -> ModuleIndex
    {
        return ModuleIndex {
            module: Subject("the/module"),
            members: vec![
                MemberReading {
                    subject: Subject("alpha.rs"),
                    outcome: Outcome::Read,
                },
                MemberReading {
                    subject: Subject("beta.rs"),
                    outcome: Outcome::Approximate,
                },
                MemberReading {
                    subject: Subject("gamma.rs"),
                    outcome: Outcome::Unreachable,
                },
            ],
            items: vec![IndexEntry {
                member: Subject("alpha.rs"),
                ordinal: 0,
                kind: "Function".to_owned(),
                visibility: "Public".to_owned(),
                qualified_name: "inner::Deep".to_owned(),
            }],
        };
    }

    #[test]
    fn Test_An_Index_Should_Survive_A_Round_Trip()
    {
        let index = An_Index();

        assert_eq!(Parse_Index(&Encode_Index(&index)), Ok(index));
    }

    /// The encoding is the fact's content address, so it must not vary with anything but
    /// the index — and it must be readable by a person, because two rollups that disagree
    /// are compared in a diff before they are compared by a tool.
    #[test]
    fn Test_The_Encoding_Should_Be_Line_Oriented_And_Local_To_Nothing()
    {
        let rendered = String::from_utf8(Encode_Index(&An_Index()))
            .expect("the encoding is ASCII tabs around hexadecimal and UTF-8 identifiers");

        assert!(!rendered.contains('\r'), "line endings must not be local");
        assert!(rendered.starts_with("module\t"));
        assert_eq!(rendered.lines().count(), 5);
    }

    /// A member that could not be read must not encode like a member that declares
    /// nothing.
    ///
    /// An index over one module and one member, differing only in what became of it.
    ///
    /// The three outcomes are the point of these tests: each pair is the same member count
    /// and a different answer, and an encoding that collapsed any two of them would let a
    /// run report a clean module it never read.
    fn One_Member(outcome: Outcome) -> ModuleIndex
    {
        return ModuleIndex {
            module: Subject("the/module"),
            members: vec![MemberReading {
                subject: Subject("alpha.rs"),
                outcome,
            }],
            items: Vec::new(),
        };
    }

    /// The two are the same number of items and different answers, and collapsing them is
    /// how a run comes to report a clean module it never read.
    #[test]
    fn Test_An_Unreachable_Member_Should_Not_Encode_Like_A_Silent_One()
    {
        let unreachable = One_Member(Outcome::Unreachable);
        let silent = One_Member(Outcome::Read);

        assert_ne!(Encode_Index(&unreachable), Encode_Index(&silent));
    }

    /// An approximated rollup must not encode like an exact one.
    ///
    /// `OD-CAPABILITY-003`'s third condition. If the distinction did not reach the bytes,
    /// buying coverage from a weaker provider would also buy the appearance of precision
    /// and nothing downstream could tell the two rollups apart.
    #[test]
    fn Test_An_Approximated_Member_Should_Not_Encode_Like_An_Exact_One()
    {
        let exact = One_Member(Outcome::Read);
        let approximated = One_Member(Outcome::Approximate);

        assert_ne!(Encode_Index(&exact), Encode_Index(&approximated));
    }

    /// The negative controls for the reader.
    ///
    /// Every one of these would decode to an empty or partial index under a reader that
    /// fell back to a default, and an empty index reads as a module that declares
    /// nothing — indistinguishable from a module that genuinely does.
    #[test]
    fn Test_Bytes_That_Are_Not_An_Index_Should_Be_Refused()
    {
        assert!(
            Parse_Index(b"").is_err(),
            "the empty payload is not a module with no members"
        );
        assert!(
            Parse_Index(b"member\t00000000000000000000000000000000\tread\n").is_err(),
            "a payload whose first record is not `module` names no subject"
        );
        assert!(
            Parse_Index(b"module\tnot-a-digest\n").is_err(),
            "a subject that is not a digest is not a subject"
        );
        assert!(
            Parse_Index(b"module\t0000000000000000000000000000000g\n").is_err(),
            "the right length and the wrong alphabet is still not a digest"
        );

        let module = format!("module\t{}\n", Subject("the/module").Digest());
        assert!(
            Parse_Index(format!("{module}{module}").as_bytes()).is_err(),
            "a second `module` record would leave two answers to which module this is"
        );
        assert!(
            Parse_Index(format!("{module}surface\t1\n").as_bytes()).is_err(),
            "an unknown record tag is most likely a newer schema, which is exactly the \
             case where guessing loses the information that was added"
        );
        assert!(
            Parse_Index(
                format!("{module}member\t{}\tmaybe\n", Subject("alpha.rs").Digest()).as_bytes()
            )
            .is_err(),
            "an outcome this build does not know is not `read`"
        );
        assert!(
            Parse_Index(
                format!("{module}item\t{}\t0\tFunction\n", Subject("alpha.rs").Digest()).as_bytes()
            )
            .is_err(),
            "an item record missing its name is not an item with no name"
        );
    }

    /// A record longer than the grammar is refused, not read down to what this build knows.
    ///
    /// The half `P10-INDEX-FIELD-STRICTNESS` was written for. Every payload here is what a
    /// v2 of this schema would plausibly look like from a v1 reader: the fields it knows,
    /// followed by one it does not. Truncating instead of refusing hands a caller a clean
    /// decode of a payload it only partly understood, and the field it dropped is the one
    /// that changed.
    #[test]
    fn Test_A_Record_With_A_Field_This_Build_Does_Not_Know_Should_Be_Refused()
    {
        let module = format!("module\t{}\n", Subject("the/module").Digest());
        let alpha = Subject("alpha.rs").Digest();

        assert!(
            Parse_Index(format!("module\t{alpha}\tv2\n").as_bytes()).is_err(),
            "a `module` record with a field after the subject is not this schema"
        );
        assert!(
            Parse_Index(format!("{module}member\t{alpha}\tread\t42\n").as_bytes()).is_err(),
            "a `member` record carrying something after its outcome is not this schema"
        );
        assert!(
            Parse_Index(
                format!("{module}item\t{alpha}\t0\tFunction\tPublic\tOne\tv2\n").as_bytes()
            )
            .is_err(),
            "an `item` record carrying an eighth field is not this schema"
        );
    }

    /// The control for the test above, and the one that stops it from being satisfied by a
    /// reader that refuses everything.
    ///
    /// A stricter reader that also refused this provider's own output would be a schema with
    /// no conforming writer, which is a worse defect than the laxness it replaced — and it
    /// would fail here rather than in whatever consumes a rollup three commits from now.
    #[test]
    fn Test_Every_Record_This_Encoder_Writes_Should_Still_Be_Accepted()
    {
        let index = An_Index();
        let encoded = Encode_Index(&index);
        let rendered = String::from_utf8(encoded.clone()).expect("the encoding is text");

        assert_eq!(Parse_Index(&encoded), Ok(index));

        // Every record form the encoder can emit is present above, so the round trip is a
        // statement about the grammar rather than about one record. A test that round-tripped
        // a payload with no `item` record would say nothing about the longest record there is.
        assert!(rendered.contains("\nmember\t"), "{rendered}");
        assert!(rendered.contains("\nitem\t"), "{rendered}");
        for outcome in [READ, APPROXIMATE, UNREACHABLE]
        {
            assert!(rendered.contains(outcome), "{outcome} is not exercised: {rendered}");
        }
    }

    /// The ceiling admits a weaker offer, which is what a ceiling is for.
    #[test]
    fn Test_The_Ceiling_Should_Admit_This_Providers_Offer()
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }

    /// And refuses one above it. The property the contract exists for, asserted where the
    /// ceiling is written.
    #[test]
    fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Resolution()
    {
        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");

        let resolved = ProviderOffer {
            guarantee: Guarantee::New(
                FactVariant::SemanticallyResolved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            ),
            ..Provider_Offer()
        };

        assert!(
            registry.Offer(resolved).is_err(),
            "an index of what files declare is a statement about what they say on their \
             face, and a provider claiming resolution would satisfy every rule that needs it"
        );
    }

    /// The ceiling is not this provider's own claim.
    ///
    /// A ceiling equal to the incumbent's guarantee has to be raised whenever somebody
    /// improves something, and silently forbids a better second provider in the meantime.
    #[test]
    fn Test_The_Ceiling_Should_Leave_Room_Above_This_Provider()
    {
        assert_ne!(Ceiling(), Declared_Guarantee());

        let mut registry = Registry::New();
        registry.Declare(Capability_Contract()).expect("declared once");
        registry.Offer(Provider_Offer()).expect("within the ceiling");

        let finer_refresh = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
        let needs_a_finer_refresh = Requirement::New(Capability(), CONTRACT_VERSION, finer_refresh);

        assert!(
            matches!(
                registry.Resolve(&needs_a_finer_refresh),
                Resolution::Unsatisfied {
                    reason: Unmet::BelowRequirement { .. },
                    ..
                }
            ),
            "a caller that must refresh one file at a time cannot be served by a rollup \
             that can only refresh a whole module, and the contract admits the provider \
             that could be"
        );
    }
}
