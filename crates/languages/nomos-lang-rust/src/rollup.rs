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

use crate::provider::{FactContext, Syntax_Inputs};
use nomos_analysis::{
    Context, Dependency, FactError, FactKey, FactPayload, FactReader, GuaranteeDigest, InputDigest,
    MaterializedFact, MemoryFactStore, Reader,
};
use nomos_capability::{CapabilityContract, ProviderOffer, Registry, Requirement};
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

/// One file a module is made of.
///
/// Carries the subject *and* the semantic inputs its syntax fact was computed from,
/// because a fact is looked up by rebuilding its key and the inputs are a component of
/// one. [`ModuleMember::Of`] is the way to construct it: it routes through
/// [`Syntax_Inputs`], so the digest this rebuilds a key with is the digest that wrote it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleMember
{
    pub subject: SubjectId,
    pub inputs: InputDigest,
}

impl ModuleMember
{
    /// A member from the file's subject and its entire contents.
    #[must_use]
    pub fn Of(subject: SubjectId, source: &str) -> Self
    {
        return Self {
            subject,
            inputs: Syntax_Inputs(source),
        };
    }
}

/// The subject a rollup is about, and the files it is over.
///
/// The module's own subject must not be any member's. A rollup keyed on one of its inputs
/// would be named by the same [`nomos_analysis::GenerationCause`] that names the input,
/// and would be reported as directly invalidated by a change it was actually reached by —
/// which is the transitive half of invalidation disappearing into the direct half.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module
{
    pub subject: SubjectId,
    pub members: Vec<ModuleMember>,
}

/// How a member's syntax fact was obtained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome
{
    /// Answered by a provider the caller's requirement admitted at full standing.
    Read,
    /// Answered by a weaker provider than the requirement asked for.
    ///
    /// `OD-CAPABILITY-003`'s third condition. Without a distinct value the scanner's
    /// answer covers the file the parser refused, `Unreachable` drops to zero, and an
    /// index over five parsed members and one pattern-matched one encodes identically to
    /// one over six parsed members.
    Approximate,
    /// No admitted provider had a readable answer for this member.
    ///
    /// Never silently dropped. A rollup that omits the members it could not read reports a
    /// smaller module as though it were a complete one.
    Unreachable,
}

impl Outcome
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Read => READ,
            Self::Approximate => APPROXIMATE,
            Self::Unreachable => UNREACHABLE,
        };
    }

    /// Whether the member contributed entries. `Approximate` did; it says how good they
    /// are, not whether they are there.
    #[must_use]
    pub const fn Answered(self) -> bool
    {
        return matches!(self, Self::Read | Self::Approximate);
    }
}

/// A member of the module, and what came of reading it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemberReading
{
    pub subject: SubjectId,
    pub outcome: Outcome,
}

/// One declared item, attributed to the member that declared it.
///
/// `qualified_name` is the syntax schema's, unchanged: a name qualified by nesting within
/// its own file, and not a resolved path. What this adds is `member`, without which two
/// files declaring the same name are one record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexEntry
{
    pub member: SubjectId,
    pub ordinal: u32,
    pub kind: String,
    pub visibility: String,
    pub qualified_name: String,
}

/// What a module declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleIndex
{
    pub module: SubjectId,
    pub members: Vec<MemberReading>,
    pub items: Vec<IndexEntry>,
}

impl ModuleIndex
{
    /// Members that contributed entries, whether exactly or approximately.
    #[must_use]
    pub fn Answered(&self) -> usize
    {
        return self
            .members
            .iter()
            .filter(|member| return member.outcome.Answered())
            .count();
    }

    /// Members whose syntax fact could not be read.
    #[must_use]
    pub fn Unreachable(&self) -> usize
    {
        return self
            .members
            .iter()
            .filter(|member| return member.outcome == Outcome::Unreachable)
            .count();
    }

    /// Members answered by a weaker provider than the caller asked for.
    #[must_use]
    pub fn Approximated(&self) -> usize
    {
        return self
            .members
            .iter()
            .filter(|member| return member.outcome == Outcome::Approximate)
            .count();
    }
}

/// A rollup that has been written to the store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rolled
{
    /// Where the derived fact is filed. A caller invalidating or re-reading it needs this.
    pub key: FactKey,
    pub index: ModuleIndex,
    /// What the reader observed, which is what the store was given as edges.
    pub dependencies: Vec<Dependency>,
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
    registry: &Registry,
    need: &Requirement,
    module: &Module,
    context: FactContext,
) -> Result<Rolled, FactError>
{
    let members = Canonical(&module.members);
    let key = Index_Key(module.subject, &members, context);

    // The read borrows the store immutably and the write needs it mutably, so the reader
    // is confined to this scope. What survives it is owned: the index, and the edges the
    // reader observed on the way to it.
    let (index, dependencies) =
        Read_Members(store, registry, need, module.subject, &members, context);

    store.Materialize(
        MaterializedFact {
            identity: key.clone().At(context.generation),
            // Provenance: the tree this rollup was computed over. Not part of the key, so
            // the workspace moving re-addresses nothing — `OD-ANALYSIS-001`.
            snapshot: context.snapshot,
            // No stronger than what it derived from. Promoting this to `Verified` would
            // launder the rollup's own arithmetic into a measurement.
            evidence: EvidenceClass::Derived,
            guarantee: Declared_Guarantee(),
            payload: FactPayload::New(Payload_Schema(), Encode_Index(&index)),
        },
        &dependencies,
    )?;

    return Ok(Rolled {
        key,
        index,
        dependencies,
    });
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
    registry: &Registry,
    need: &Requirement,
    module: SubjectId,
    members: &[ModuleMember],
    context: FactContext,
) -> (ModuleIndex, Vec<Dependency>)
{
    let mut reader = Reader::On(
        store,
        registry,
        Context {
            snapshot: context.snapshot,
            variant: context.variant,
            configuration: context.configuration,
            generation: context.generation,
        },
    );
    let capability = nomos_cap_syntax::Capability();
    let mut index = ModuleIndex {
        module,
        members: Vec::new(),
        items: Vec::new(),
    };

    for member in members
    {
        // `Require_Any` rather than `Require`: reading only the chosen provider leaves a
        // weaker provider's answer written into the store and unread, and the rollup
        // reports the member missing while the answer sits one lookup away.
        let read = reader.Require_Any(&capability, &member.subject, member.inputs, need);

        let Ok((fact, applicability)) = read
        else
        {
            index.members.push(MemberReading {
                subject: member.subject,
                outcome: Outcome::Unreachable,
            });
            continue;
        };

        let decoded = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes);

        let Ok(payload) = decoded
        else
        {
            // Bytes filed under the syntax schema that are not the syntax schema. Counted
            // unreachable rather than as an empty answer: a member that could not be read
            // must not encode like a member that declares nothing.
            index.members.push(MemberReading {
                subject: member.subject,
                outcome: Outcome::Unreachable,
            });
            continue;
        };

        index.members.push(MemberReading {
            subject: member.subject,
            outcome: if applicability == Applicability::SupportedWithFallback
            {
                Outcome::Approximate
            }
            else
            {
                Outcome::Read
            },
        });

        for item in payload.items
        {
            index.items.push(IndexEntry {
                member: member.subject,
                ordinal: item.ordinal,
                kind: item.kind,
                visibility: item.visibility,
                qualified_name: item.qualified_name,
            });
        }
    }

    return (index, reader.Into_Dependencies());
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

    return encoded.into_bytes();
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
    let text = core::str::from_utf8(payload)
        .map_err(|error| return format!("the payload is not UTF-8, so it is not this schema: {error}"))?;

    let mut lines = text.lines().enumerate();

    let Some((_, header)) = lines.next()
    else
    {
        return Err("the payload is empty, which is not a module that declares nothing — \
                    that is a `module` record and no members"
            .to_owned());
    };

    let Some(module) = header.strip_prefix("module\t")
    else
    {
        return Err(format!("the first record is `{header}` and not a `module` record"));
    };
    let module = Subject_From(module.trim(), 1)?;

    let mut index = ModuleIndex {
        module,
        members: Vec::new(),
        items: Vec::new(),
    };

    for (offset, line) in lines
    {
        let number = offset.saturating_add(1);
        let mut fields = line.split('\t');

        match fields.next()
        {
            Some("module") => return Err(format!("line {number} is a second `module` record")),
            Some("member") => index.members.push(Member_Record(&mut fields, number)?),
            Some("item") => index.items.push(Item_Record(&mut fields, number)?),
            Some(tag) => return Err(format!("line {number} has record tag `{tag}`, which this build does not understand")),
            None => return Err(format!("line {number} is empty")),
        }
    }

    return Ok(index);
}

fn Member_Record<'text>(
    fields: &mut impl Iterator<Item = &'text str>,
    line: usize,
) -> Result<MemberReading, String>
{
    let (Some(subject), Some(outcome)) = (fields.next(), fields.next())
    else
    {
        return Err(format!("line {line} is a `member` record without a subject and an outcome"));
    };

    let outcome = match outcome
    {
        READ => Outcome::Read,
        APPROXIMATE => Outcome::Approximate,
        UNREACHABLE => Outcome::Unreachable,
        other => return Err(format!("`{other}` on line {line} is not an outcome")),
    };

    return Ok(MemberReading {
        subject: Subject_From(subject, line)?,
        outcome,
    });
}

fn Item_Record<'text>(
    fields: &mut impl Iterator<Item = &'text str>,
    line: usize,
) -> Result<IndexEntry, String>
{
    let (Some(member), Some(ordinal), Some(kind), Some(visibility), Some(qualified_name)) = (
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
    )
    else
    {
        return Err(format!("line {line} is an `item` record with fewer than five fields"));
    };

    let ordinal: u32 = ordinal
        .parse()
        .map_err(|_| return format!("`{ordinal}` on line {line} is not an ordinal"))?;

    return Ok(IndexEntry {
        member: Subject_From(member, line)?,
        ordinal,
        kind: kind.to_owned(),
        visibility: visibility.to_owned(),
        qualified_name: qualified_name.to_owned(),
    });
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
    /// The two are the same number of items and different answers, and collapsing them is
    /// how a run comes to report a clean module it never read.
    #[test]
    fn Test_An_Unreachable_Member_Should_Not_Encode_Like_A_Silent_One()
    {
        let unreachable = ModuleIndex {
            module: Subject("the/module"),
            members: vec![MemberReading {
                subject: Subject("alpha.rs"),
                outcome: Outcome::Unreachable,
            }],
            items: Vec::new(),
        };
        let silent = ModuleIndex {
            members: vec![MemberReading {
                subject: Subject("alpha.rs"),
                outcome: Outcome::Read,
            }],
            ..unreachable.clone()
        };

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
        let exact = ModuleIndex {
            module: Subject("the/module"),
            members: vec![MemberReading {
                subject: Subject("alpha.rs"),
                outcome: Outcome::Read,
            }],
            items: Vec::new(),
        };
        let approximated = ModuleIndex {
            members: vec![MemberReading {
                subject: Subject("alpha.rs"),
                outcome: Outcome::Approximate,
            }],
            ..exact.clone()
        };

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

        let needs_a_finer_refresh = Requirement::New(
            Capability(),
            CONTRACT_VERSION,
            Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Unknown,
                IncrementalGranularity::File,
            ),
        );

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
