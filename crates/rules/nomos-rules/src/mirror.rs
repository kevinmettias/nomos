//! The completeness-mirror rule.
//!
//! `OD-COMPLETENESS-001` names the shape and `P10-FIRST-CHECK` asks for one rule that
//! runs end to end and judges real code. This is that rule, and it is the one chosen
//! because it is the only rule in this tree with three recorded historical instances to
//! test a judgment against.
//!
//! # What it judges
//!
//! A declared universe must be mirrored by a check that compares the declaration against
//! the reality it claims to enumerate. The universe declares that check by name, at the
//! site, and this rule resolves the name against the real source.
//!
//! Three outcomes, and the ordering between the last two is the whole point:
//!
//! | The universe | The rule says |
//! |---|---|
//! | names a check that exists | nothing — it is mirrored |
//! | names a check that does not exist | a **blocking** finding |
//! | names nothing | an **advisory** finding |
//!
//! A false claim of coverage is worse than an admitted gap. `enforcement.rs` already
//! says so about enforcers — a phantom is "worse than declaring no enforcer at all:
//! nothing runs, nothing can fail, and the declaration says the rule is covered so no
//! reader looks twice" — and the same asymmetry is why this rule blocks on one and
//! reports on the other. It is also what keeps the check green today: this workspace has
//! twelve unmirrored universes, and a gate that can never be green is a gate everybody
//! learns to ignore.
//!
//! There is a third outcome above both, added by `OD-RULES-001`: **the rule saying it
//! could not answer.** A subject whose syntax fact could not be read leaves the check
//! index short, and a claimed mirror the shortfall could have accounted for is reported at
//! the reader's [`Applicability`] rather than as an [`EnforcementBreach::Phantom`] —
//! because the rule cannot tell a false claim of coverage from a name it did not get to
//! look for. The asymmetry `D-134` decided is untouched; what is new is that absence no
//! longer has to be squeezed into one of its two arms.
//!
//! `OD-RULES-002` scopes that third outcome to the claims it actually bears on. Under
//! `OD-RULES-001` incompleteness was one flag over the whole run, so one unreadable file
//! anywhere silenced every phantom in the tree — including phantoms claimed in files whose
//! facts were read perfectly well, and whose claims the unreadable file could not have
//! resolved. That is a guard that reports rather than judges, and over this workspace it
//! was permanent: `tests/corpus/analysis/gamma/broken.rs` is a fixture the parser is
//! *supposed* to refuse, so the flag was set on every run. The downgrade stays and its
//! reasoning stays; what changes is that the index is asked whether it is short **of
//! something that could have resolved this name**, and the test for that is
//! [`Unread::Could_Have_Declared`].
//!
//! # Where the check names come from
//!
//! From `nomos.cap.syntax.items` facts, one [`nomos_analysis::FactReader::Require`] per
//! subject, under the floor [`crate::Syntax_Requirement`] declares. Not from a parser
//! vendored here: `D-134` created this workspace's second `syn` front end and recorded a
//! reason — replayability — that proves a rule takes its subject as an argument and does
//! not prove that the argument is text. `OD-RULES-001` withdraws the inference and moves
//! this half of the rule onto the fact layer. Universe discovery keeps its parser, for a
//! measured reason `universe.rs` states.
//!
//! What that buys is not caching and not incrementality; this rule spends neither. It
//! spends [`nomos_contracts::Guarantee::Satisfies`]: the floor is the rule's own, a
//! composition root cannot lower it, and the difference between a parser and a line
//! scanner now has a verdict attached to it.
//!
//! # Why the judgment is built on `EnforcementReach`
//!
//! Because it already exists and it is already right. `declared`, `expected` and
//! `computed` are exactly the three things this rule has — what the universe names, what
//! naming it amounts to as a claim, and what the source says it really amounts to — and
//! [`EnforcementReach::Is_Enforced`] and [`EnforcementReach::Is_Truthful`] are already
//! the two questions being asked. A second judgment written next to it would be a second
//! place for the same rule to be spelled, which is how two guards for one rule come to
//! disagree.

use crate::facts::Check_Names_In;
use crate::universe::{DeclaredUniverse, Reading, Read_Universes, UniverseKind};
use crate::{SourceFile, Syntax_Requirement};
use nomos_analysis::{FactReader, InputDigest};
use nomos_contracts::{
    Applicability, EnforcementBreach, EnforcementReach, EnforcerRef, EvidenceClass, Finding,
    GateCategory, RuleId, SubjectId,
};
use nomos_model::Content_Digest;
use std::collections::BTreeSet;

/// The rule's stable identifier.
pub const COMPLETENESS_MIRROR: &str = "completeness-mirror";

/// Judges every declared universe in `sources`, resolving claimed mirrors against facts.
///
/// `facts` is the second half of the subject and not a service the rule reaches out to.
/// It is handed in for the same reason `sources` is: a test composes one over three files
/// that no longer exist anywhere and gets the same judgment the binary gets over the real
/// tree. There is one signature and not two — a text-only entry point kept beside this one
/// would be a second place for one rule to be spelled, and it would let a shipped binary
/// consult no fact while the tests all did.
///
/// Findings come back sorted by subject name, which is the stable one. Sorting by path
/// would reorder the whole report when a file moves, and a report that reorders is a
/// report nobody can diff.
#[must_use]
pub fn Check_Completeness_Mirrors(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let index = Check_Index_Of(sources, facts);

    let mut universes: Vec<DeclaredUniverse> = Vec::new();
    let mut findings: Vec<Finding> = Vec::new();

    for source in sources
    {
        match Read_Universes(&source.path, &source.text)
        {
            Reading::Parsed(found) => universes.extend(found),
            // A file this rule could not read is reported, not skipped. Skipping it would
            // fold "there is nothing here" into "I could not look", which is the one
            // conflation `Applicability` exists to prevent.
            Reading::Unparseable { because } => findings.push(Unreadable(source, &because)),
        }
    }

    // One finding per subject whose fact could not be read, before any universe is
    // judged. A run that materialized nothing must not be able to render as a clean tree,
    // and that property has to hold whether or not the tree happens to declare a universe.
    findings.extend(index.unread.iter().map(Unread_Subject));

    universes.sort();
    universes.dedup();

    findings.extend(
        universes
            .iter()
            .filter_map(|universe| return Judge(universe, &index)),
    );
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));

    return findings;
}

/// A file the rule could not read.
///
/// Identified by the digest of its contents rather than by its name, because what could
/// not be parsed is this text — the same path holding different bytes is a different
/// fact, and a finding keyed on the name would look unchanged after an edit that fixed
/// it.
fn Unreadable(source: &SourceFile, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Content_Digest(source.text.as_bytes())),
        subject_name: source.path.clone(),
        applicability: Applicability::Unparseable,
        evidence: EvidenceClass::Derived,
        // Advisory, and it would make no difference if it were not: `Can_Fail_A_Build`
        // consults the applicability too, and a rule that never read its subject must not
        // stop anybody. What this finding is for is being *counted* — a run that could not
        // read four files and reports clean is the defect, not the four files.
        gate: GateCategory::Advisory,
        summary: format!("could not be parsed, so no universe in it was judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

/// What one universe's declaration amounts to, and what is really true of it.
///
/// Returns `None` when the universe is mirrored, because a rule that emits a finding per
/// subject it approves of produces a report in which the defects cannot be found.
fn Judge(universe: &DeclaredUniverse, index: &CheckIndex<'_>) -> Option<Finding>
{
    let reach = Reach_Of(universe, &index.names);

    if reach.Is_Enforced()
    {
        return None;
    }

    let (applicability, gate, summary) = match reach.breaches.first()
    {
        // The name is read back off the universe rather than off the breach because the
        // breach's payload is `nomos-contracts`' shape and this is the rule's own claim.
        // The two cannot disagree: `Reach_Of` produces a breach only in the arm where
        // `claimed_mirror` is `Some`, and produces none in the arm where it is `None`, so
        // the `unwrap_or_default` below is unreachable rather than a fallback with a
        // meaning. An empty name matches every text, which would downgrade rather than
        // block — the safe direction, for the reason [`Unread::Could_Have_Declared`]
        // gives.
        Some(breach) => Unresolved_Claim(
            breach,
            universe.claimed_mirror.as_deref().unwrap_or_default(),
            index,
        ),
        None => Admitted_Gap(universe),
    };

    return Some(Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Content_Digest(universe.name.as_bytes())),
        subject_name: universe.name.clone(),
        // An admitted gap does not depend on the index at all — nothing was resolved, so
        // nothing could have been missed — and stays `Supported` however short the index
        // is. Only a claim that failed to resolve inherits the doubt.
        applicability,
        // Computed from source by a deterministic rule, and no stronger than that source.
        evidence: EvidenceClass::Derived,
        gate,
        summary,
        locations: vec![universe.path.clone()],
    });
}

/// How to report a claimed mirror that did not resolve against the index.
///
/// The two gates in play are not the same gate, and collapsing them is the mistake this
/// whole module is about. `reach.computed` is what the *universe's* declared mirror amounts
/// to — `Unreachable` for a phantom. The gate returned here is what *this rule* does about
/// that, and a false claim of coverage is the one outcome worth failing a build over.
///
/// Unless the index is short of something that could have resolved *this* name. Then the
/// claim is not established as false — the name may be in the subject that was not read —
/// and reporting it as a phantom would be the rule manufacturing the one finding it is
/// entitled to stop a build over out of its own inability to look, which is the same defect
/// as reporting clean wearing the other face. `Can_Fail_A_Build` consults the applicability
/// as well as the gate, so the refusal is machinery `D-134` already built rather than a
/// second rule about severity.
///
/// `OD-RULES-001` asked that question of the whole run and this asks it of the claim, which
/// is the whole of `OD-RULES-002`. When the index is short of subjects that *could not*
/// have resolved this name, the judgment is made and the shortfall travels with it in the
/// summary rather than suppressing it: a reader who wants to know what the run did not see
/// is owed that on the finding, not instead of it.
fn Unresolved_Claim(
    breach: &EnforcementBreach,
    claimed: &str,
    index: &CheckIndex<'_>,
) -> (Applicability, GateCategory, String)
{
    let Some(shortfall) = index.Shortfall_For(claimed)
    else
    {
        return (
            Applicability::Supported,
            GateCategory::Blocking,
            if index.unread.is_empty()
            {
                breach.Describe()
            }
            else
            {
                format!(
                    "{} — and the check index is short {} subject(s), none of whose text \
                     spells `{claimed}`, so no reading of them could have declared it",
                    breach.Describe(),
                    index.unread.len()
                )
            },
        );
    };

    return (
        shortfall.Applicability(),
        GateCategory::Advisory,
        format!("{} — and {}", breach.Describe(), shortfall.Describe()),
    );
}

/// How to report a universe that claims no mirror at all.
///
/// An admitted gap does not depend on the index in any way — nothing was resolved, so
/// nothing could have been missed — which is why no shortfall is consulted here.
fn Admitted_Gap(universe: &DeclaredUniverse) -> (Applicability, GateCategory, String)
{
    return (
        Applicability::Supported,
        GateCategory::Advisory,
        format!(
            "declares no mirror, so nothing compares this list against the reality it \
             enumerates; a {} added without adding it here is outside every guard built \
             on it, and those guards then pass by not looking",
            match universe.kind
            {
                UniverseKind::Constant => "member",
                UniverseKind::Enumeration => "variant",
            }
        ),
    );
}

/// The enforcement claim a universe makes, and what the source says of it.
///
/// A universe that names no mirror declares [`EnforcerRef::Review`] and expects
/// [`GateCategory::Review`], which is *truthful* — an admitted gap is an honest
/// declaration, and `enforcement.rs` is explicit that it must not be conflated with an
/// overclaim. It is still not enforcement, which is why the finding is raised on
/// [`EnforcementReach::Is_Enforced`] and its severity read off the breaches.
fn Reach_Of(universe: &DeclaredUniverse, checks: &BTreeSet<String>) -> EnforcementReach
{
    let Some(claimed) = universe.claimed_mirror.as_ref()
    else
    {
        return EnforcementReach {
            rule: RuleId::New(COMPLETENESS_MIRROR),
            declared: vec![EnforcerRef::Review],
            expected: GateCategory::Review,
            computed: GateCategory::Review,
            breaches: Vec::new(),
        };
    };

    let enforcer = EnforcerRef::Check {
        name: claimed.clone(),
    };
    let resolves = checks.contains(claimed);

    return EnforcementReach {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        declared: vec![enforcer],
        // Naming a check is a claim that a violation would be caught. That is what
        // makes a name that resolves to nothing a false claim rather than a typo.
        expected: GateCategory::Blocking,
        computed: if resolves
        {
            GateCategory::Blocking
        }
        else
        {
            GateCategory::Unreachable
        },
        breaches: if resolves
        {
            Vec::new()
        }
        else
        {
            vec![EnforcementBreach::Phantom {
                name: claimed.clone(),
            }]
        },
    };
}

/// One subject whose check names could not be read, and why not.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Unread<'source>
{
    /// Where to look, for the reader. Reporting only.
    path: String,
    /// The subject's own text, borrowed from the sources the rule was handed.
    ///
    /// Not a second source of check names, and it must never become one — the floor
    /// [`crate::Syntax_Requirement`] states exists to stop exactly that, and
    /// [`Unread::Could_Have_Declared`] is the only thing that reads this field.
    text: &'source str,
    /// The digest of the text the answer was asked about.
    ///
    /// The finding's identity. Not the [`SubjectId`] the fact was requested under: that is
    /// a digest of a path, and `identity.rs` states the rule for the whole system — a
    /// finding keyed on where something lives closes and reopens every time somebody moves
    /// a file. What could not be read is *these bytes*, which is what [`Unreadable`] keys
    /// on for the same reason one paragraph up.
    inputs: SubjectId,
    /// What the reader said, in the vocabulary a run reports in.
    applicability: Applicability,
    /// The specific reason, for the summary.
    because: String,
}

impl Unread<'_>
{
    /// Whether any reading of this subject could have declared `name`.
    ///
    /// This is the whole of `OD-RULES-002`, so it is worth being exact about what it claims
    /// and in which direction it is allowed to be wrong.
    ///
    /// A check name in the index is an identifier the provider read out of a token stream,
    /// and an identifier in a token stream is spelled in the bytes the stream was lexed
    /// from. So a subject whose text does not contain `name` as a substring cannot have
    /// declared it, whatever the provider would have said about the rest of the file. That
    /// direction is sound, and it is the direction this function is used in: it decides when
    /// the index is short **of something that could have resolved this claim**.
    ///
    /// The converse is not claimed and is not needed. A substring match means only that the
    /// name is spelled somewhere in the file — in a comment, in a string literal, inside a
    /// `Test_X_And_More` — and every one of those is a false positive on "could have
    /// declared it". Each of those false positives *withholds* a block. That is why an
    /// unsound text test is admissible here and inadmissible thirty lines up:
    /// `Check_Index_Of` uses a fact for the check names because an unsound *positive* there
    /// resolves a claim and silences the rule, which is the defect this rule exists to
    /// find. Used only to add doubt, the same signal cannot manufacture a phantom — it can
    /// only fail to establish one, and `Test_A_Name_Spelled_In_A_Comment_Should_Still_\
    /// Withhold_The_Block` pins the shape.
    ///
    /// The one way this can be wrong in the *unsafe* direction is a name no reading of the
    /// bytes spells: an identifier a macro composed, or one behind an `include!`. That hole
    /// is not opened here. It is `Assurance::Unknown` completeness, which
    /// [`crate::Syntax_Requirement`] already declares and this rule already blocks under — a
    /// macro-generated `Test_X` in a *readable* file is equally absent from the index and
    /// equally reported as a phantom today. The guarantee is therefore exact with respect to
    /// what the source spells and no better, which is the same bound the rest of the rule
    /// carries rather than a new one.
    fn Could_Have_Declared(&self, name: &str) -> bool
    {
        return self.text.contains(name);
    }
}

/// Why a claim that failed to resolve is not thereby established false.
///
/// Two shapes and not one flag, because they are different sentences with different
/// remedies, and because only one of them is a statement about the claim.
enum Shortfall<'index>
{
    /// Nothing admitted could answer for any subject, so there is no index at all.
    ///
    /// Not a claim about this name: a name cannot be established absent from an index that
    /// was never built. This one is whole-run by construction rather than by policy —
    /// `Resolution::Applicability` reads the registry and the requirement, and neither
    /// varies by subject, so `MissingCapability` holds for every subject of a run or for
    /// none of them. The remedy is registering or installing a provider, which is why it
    /// must not be reported as the narrower "the store had nothing for this subject".
    NoIndex,
    /// The index is short at least one subject whose own text spells the claimed name.
    Withheld
    {
        /// What the reader said about the first of them.
        ///
        /// The first in source order, and the summary names all of them. The two values
        /// that can appear here — `DependencyUnavailable` and `Unparseable` — are both
        /// statements about one file, so neither outranks the other and a deterministic pick
        /// is the honest one. `MissingCapability` cannot appear: it is
        /// [`Shortfall::NoIndex`].
        applicability: Applicability,
        /// Where they are, for the reader.
        subjects: Vec<&'index str>,
    },
}

impl Shortfall<'_>
{
    /// How a claim this shortfall bears on must be reported.
    fn Applicability(&self) -> Applicability
    {
        return match *self
        {
            Self::NoIndex => Applicability::MissingCapability,
            Self::Withheld { applicability, .. } => applicability,
        };
    }

    /// Why the claim is not established false, in terms somebody can act on.
    ///
    /// Names the subjects rather than counting them. `OD-RULES-002`'s `done_when` asks the
    /// record to state how the scoping is decided; a finding that said only "the index is
    /// incomplete" would leave the reader unable to check that decision against the tree.
    fn Describe(&self) -> String
    {
        return match self
        {
            Self::NoIndex => "no admitted provider could answer for any subject, so there is \
                              no check index for this name to be absent from and this rule \
                              cannot tell a false claim of coverage from a name it did not \
                              get to look for"
                .to_owned(),
            Self::Withheld { subjects, .. } => format!(
                "the check index is short {}, whose text spells this name, so this rule \
                 cannot tell a false claim of coverage from a name it did not get to look for",
                subjects.join(", ")
            ),
        };
    }
}

/// Every check name in scope for this run, and every subject that is missing from it.
///
/// The two fields are one value because they are one claim. A set of names alone cannot
/// say whether it is the whole set, and a rule that treats a short index as a complete one
/// reports a name it never looked for as a name that does not exist.
struct CheckIndex<'source>
{
    /// Check names, from the facts that were read.
    names: BTreeSet<String>,
    /// Subjects whose facts were not read, in source order.
    unread: Vec<Unread<'source>>,
}

impl CheckIndex<'_>
{
    /// Whether this index is short of anything that could have resolved `claimed`, and what
    /// to report if it is.
    ///
    /// `None` means the index is not short *for this name* — either nothing is missing from
    /// it, or what is missing could not have contained the name. Both are the same statement
    /// about the claim, which is the statement the caller needs: the rule looked wherever the
    /// name could have been and it was not there.
    ///
    /// This is the function `OD-RULES-002` replaces a whole-run flag with. The counter it
    /// has to answer is that any unread file might have held the check, and the answer is
    /// that "any" is doing the work: a file whose bytes do not spell the name held nothing
    /// named that. [`Unread::Could_Have_Declared`] states the bound and where it stops.
    fn Shortfall_For(&self, claimed: &str) -> Option<Shortfall<'_>>
    {
        if self
            .unread
            .iter()
            .any(|subject| return subject.applicability == Applicability::MissingCapability)
        {
            return Some(Shortfall::NoIndex);
        }

        let bearing: Vec<&Unread<'_>> = self
            .unread
            .iter()
            .filter(|subject| return subject.Could_Have_Declared(claimed))
            .collect();

        let first = bearing.first()?;

        return Some(Shortfall::Withheld {
            applicability: first.applicability,
            subjects: bearing
                .iter()
                .map(|subject| return subject.path.as_str())
                .collect(),
        });
    }
}

/// Reads one syntax fact per source and collects the check names they declare.
///
/// A check is a test function by this workspace's naming convention: `fn Test_…`. It comes
/// from a fact rather than from a parse here, and the floor
/// [`crate::Syntax_Requirement`] states is what keeps that safe — a `fn Test_X` written
/// inside a fixture string or a block comment would resolve a claim that nothing actually
/// checks, which is the precise defect this rule exists to find arriving through the rule's
/// own back door. A line scanner reports those; a parser does not; the floor admits only
/// the second.
///
/// The semantic inputs are recomputed here from the text rather than carried on
/// [`SourceFile`]. If they disagreed with what the provider wrote, the lookup would miss —
/// loudly, as an unread subject — and a shared helper would make that class of mismatch
/// untestable.
fn Check_Index_Of<'source>(
    sources: &'source [SourceFile],
    facts: &mut dyn FactReader,
) -> CheckIndex<'source>
{
    let need = Syntax_Requirement();
    let capability = nomos_cap_syntax::Capability();
    let schema = nomos_cap_syntax::Payload_Schema();

    let mut index = CheckIndex {
        names: BTreeSet::new(),
        unread: Vec::new(),
    };

    for source in sources
    {
        let inputs = InputDigest::Of(&[source.text.as_bytes()]);

        // The outcome is reduced to owned values inside the match, because the fact is
        // borrowed from the reader and the next subject needs the reader back.
        let outcome = match facts.Require(&capability, &source.subject, inputs, &need)
        {
            Ok(fact) if fact.payload.schema != schema => Err(format!(
                "the fact for this file carries payload schema `{}`, which this build does \
                 not read",
                fact.payload.schema
            )),
            Ok(fact) => Check_Names_In(&fact.payload.bytes)
                .map_err(|refusal| return refusal.Describe()),
            Err(applicability) =>
            {
                index.unread.push(Unread {
                    path: source.path.clone(),
                    text: &source.text,
                    inputs: SubjectId::From_Digest(Content_Digest(source.text.as_bytes())),
                    applicability,
                    because: format!(
                        "no admitted provider answered for it ({})",
                        applicability.Label()
                    ),
                });
                continue;
            }
        };

        match outcome
        {
            Ok(names) => index.names.extend(names),
            // A payload this build cannot read is not an empty payload. Folding it into
            // one would make a fact nobody could decode indistinguishable from a file that
            // declares no checks, and the second is a real answer.
            Err(because) => index.unread.push(Unread {
                path: source.path.clone(),
                text: &source.text,
                inputs: SubjectId::From_Digest(Content_Digest(source.text.as_bytes())),
                applicability: Applicability::Unparseable,
                because,
            }),
        }
    }

    return index;
}

/// A subject whose check names are missing from the index.
///
/// Advisory, and it would make no difference if it were not: `Can_Fail_A_Build` consults
/// the applicability too, and every value this finding can carry says the rule did not
/// reach its subject. What this finding is for is being *counted* — a run that could not
/// read a fact for four files and reports clean is the defect, not the four files.
///
/// A file the text side also failed to parse produces two findings rather than one, and
/// that is deliberate. Two independent readings of the same file both refused, which is a
/// different observation from either one alone — and while `nomos-rules` keeps a `syn`
/// front end of its own, the case where exactly one of the two refuses is the case worth
/// being able to see.
fn Unread_Subject(subject: &Unread<'_>) -> Finding
{
    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: subject.inputs,
        subject_name: subject.path.clone(),
        applicability: subject.applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "no syntax fact could be read for this file, so any check it declares is \
             missing from the index every mirror claim is resolved against: {}",
            subject.because
        ),
        locations: vec![subject.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{
        Context, FactKey, FactPayload, GuaranteeDigest, MaterializedFact, MemoryFactStore, Reader,
    };
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_contracts::{
        Assurance, BuildVariantId, ConfigurationId, Digest128, FactVariant, GenerationId, Guarantee,
        IncrementalGranularity, ProviderId, SchemaId, SnapshotId,
    };

    /// A stand-in for a parser, at exactly the guarantee `nomos-lang-rust` declares.
    ///
    /// Named rather than imported. `nomos-rules` must not depend on a provider even in
    /// test code — that is the edge the registry exists to remove, and a rule crate that
    /// names one has answered the question the floor is supposed to ask. The end-to-end
    /// path with the real provider belongs in the composition root, and `nomos-cli`'s
    /// tests are where it is asserted.
    const PARSER: &str = "nomos.test.parses";

    /// A stand-in for a line scanner, at exactly the guarantee `nomos-lang-rust-scan`
    /// declares. Used only to show that this rule's floor refuses it.
    const SCANNER: &str = "nomos.test.scans";

    fn Parser_Guarantee() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    fn Scanner_Guarantee() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Approximate,
            Assurance::Unsound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    fn Offer(provider: &str, guarantee: Guarantee) -> ProviderOffer
    {
        return ProviderOffer {
            provider: ProviderId::New(provider),
            capability: nomos_cap_syntax::Capability(),
            version: nomos_cap_syntax::CONTRACT_VERSION,
            guarantee,
        };
    }

    /// A source file, with the subject a composition root would file its facts under.
    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(
            path,
            SubjectId::From_Digest(Content_Digest(path.as_bytes())),
            text,
        );
    }

    /// The payload a parser would write for a file declaring these functions.
    ///
    /// Hand-written against the `nomos.syntax.items.v1` encoding rather than produced by
    /// calling a provider, for the reason `PARSER` gives. Every name is emitted qualified
    /// by a `tests` module, because that is the shape every check in this workspace really
    /// has and a reader that only handled bare names would pass here and resolve nothing
    /// in the product.
    fn Payload(declared: &[&str]) -> Vec<u8>
    {
        let mut encoded = String::from("unexpanded\t0\n");

        for (ordinal, name) in declared.iter().enumerate()
        {
            encoded.push_str("item\t");
            encoded.push_str(&ordinal.to_string());
            encoded.push_str("\tFunction\tPrivate\ttests::");
            encoded.push_str(name);
            encoded.push('\n');
        }

        return encoded.into_bytes();
    }

    /// The parts of a composition a rule reads through: a registry and a store.
    ///
    /// Assembled here rather than imported from anywhere, which is the property `D-134`
    /// was built on and this record claims is unaffected — a test builds the whole subject
    /// out of files that no longer exist, and now out of facts about them too.
    struct World
    {
        store: MemoryFactStore,
        registry: Registry,
    }

    impl World
    {
        /// A registry holding the syntax contract and whichever offers are named.
        fn Offering(providers: &[(&str, Guarantee)]) -> Self
        {
            let mut registry = Registry::New();
            registry
                .Declare(nomos_cap_syntax::Capability_Contract())
                .expect("the syntax capability is declared once");

            for (provider, guarantee) in providers
            {
                registry
                    .Offer(Offer(provider, *guarantee))
                    .expect("every offer here is within the capability's ceiling");
            }

            return Self {
                store: MemoryFactStore::New(),
                registry,
            };
        }

        fn Context() -> Context
        {
            return Context {
                snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
                variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
                configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
                generation: GenerationId::INITIAL,
            };
        }

        /// The key a named offer's answer about one source would be filed under.
        ///
        /// Assembled from the offer rather than from the provider's name alone, because
        /// `Reader::Require` derives the same key from whichever offer the registry chose.
        /// A test that filed a fact under a key the reader does not compute would assert
        /// that the rule reports every subject unread, which is a green test and no
        /// coverage at all.
        fn Key(source: &SourceFile, offer: &ProviderOffer) -> FactKey
        {
            let context = Self::Context();

            return FactKey {
                contract: nomos_cap_syntax::Capability(),
                contract_version: offer.version,
                subject: source.subject,
                semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
                provider: offer.provider.clone(),
                provider_version: offer.version,
                guarantee: GuaranteeDigest::Of(&offer.guarantee),
                variant: context.variant,
                configuration: context.configuration,
            };
        }

        /// Files a fact about one source, exactly as a provider registered here would.
        fn Materializing(mut self, source: &SourceFile, schema: &str, payload: Vec<u8>) -> Self
        {
            let context = Self::Context();
            let offer = Offer(PARSER, Parser_Guarantee());

            self.store
                .Materialize(
                    MaterializedFact {
                        identity: Self::Key(source, &offer).At(context.generation),
                        snapshot: context.snapshot,
                        evidence: EvidenceClass::Verified,
                        guarantee: offer.guarantee,
                        payload: FactPayload::New(SchemaId::New(schema), payload),
                    },
                    &[],
                )
                .expect("nothing here is backdated");

            return self;
        }

        fn Reader(&self) -> Reader<'_, '_>
        {
            return Reader::On(&self.store, &self.registry, Self::Context());
        }
    }

    /// A parser is registered, and it answered for every source named here.
    ///
    /// What a file "declares" is stated by the test rather than parsed out of the fixture,
    /// which is the honest shape: this crate does not parse for check names any more, and
    /// a test that derived the payload from the text would be asserting a parser it does
    /// not own.
    fn World_Over(declaring: &[(&SourceFile, &[&str])]) -> World
    {
        let mut world = World::Offering(&[(PARSER, Parser_Guarantee())]);

        for (source, declared) in declaring
        {
            world = world.Materializing(source, nomos_cap_syntax::SCHEMA, Payload(declared));
        }

        return world;
    }

    /// Findings over sources whose facts declare no checks at all.
    ///
    /// The common case for a universe test: the tree is read whole, and nothing in it
    /// mirrors anything.
    fn Findings_Over(sources: &[SourceFile]) -> Vec<Finding>
    {
        let declaring: Vec<(&SourceFile, &[&str])> = sources
            .iter()
            .map(|source| return (source, &[] as &[&str]))
            .collect();
        let world = World_Over(&declaring);
        let mut reader = world.Reader();

        return Check_Completeness_Mirrors(sources, &mut reader);
    }

    fn Only(findings: &[Finding]) -> &Finding
    {
        assert_eq!(findings.len(), 1, "expected exactly one finding: {findings:?}");
        return findings.first().expect("just asserted the length is one");
    }

    /// ---- the three instances `OD-COMPLETENESS-001` analyses ----
    ///
    /// None of the three can be replayed from git; each was repaired at the site. They
    /// are reproduced here as they were originally written, which is the whole reason
    /// this rule takes its subject as an argument rather than reading the tree. That the
    /// subject is now a pair — text and a reader — is the amendment `OD-RULES-001` makes
    /// to `D-134`, and these tests are the property it claims is unaffected. If they could
    /// not be expressed against a reader, the amendment would be wrong.
    ///
    /// Instance one and two: `Table::All`, a list kept beside the enum, over which
    /// `Assert_Complete` and `Assert_Landed` both quantified. A migration adding a table
    /// without adding it here left that table outside both guards.
    #[test]
    fn Test_The_Table_Universe_As_Originally_Written_Should_Be_Found_Unmirrored()
    {
        let findings = Findings_Over(&[Source(
            "crates/spec/nomos-spec-store/src/store.rs",
            "impl Table\n\
             {\n\
             \x20   /// Every table in the schema.\n\
             \x20   pub const fn All() -> &'static [Self]\n\
             \x20   {\n\
             \x20   }\n\
             }\n",
        )]);

        let finding = Only(&findings);

        assert_eq!(finding.subject_name, "Table::All");
        assert_eq!(finding.rule.As_Str(), COMPLETENESS_MIRROR);
        assert!(finding.summary.contains("declares no mirror"), "{}", finding.summary);
    }

    /// Instance three: `GOVERNING_RECORD_IDS`, compared against a store seeded from
    /// `GOVERNING_RECORD_IDS` — a comparison that cannot fail. Six governing records sat
    /// outside it for months.
    #[test]
    fn Test_The_Governing_Universe_As_Originally_Written_Should_Be_Found_Unmirrored()
    {
        let findings = Findings_Over(&[Source(
            "crates/spec/nomos-spec-store/src/governing.rs",
            "/// The records this store seeds itself with.\n\
             pub const GOVERNING_RECORD_IDS: &[&str] = &[\n\
             \x20   \"ARC-SPECDB-001\",\n\
             ];\n",
        )]);

        assert_eq!(Only(&findings).subject_name, "GOVERNING_RECORD_IDS");
    }

    /// And all three together, in one run, because `P10-FIRST-CHECK` asks for the rule
    /// to fail on all three rather than on each in isolation.
    #[test]
    fn Test_All_Three_Historical_Instances_Should_Fail_This_Rule()
    {
        let findings = Findings_Over(&[
            Source(
                "crates/spec/nomos-spec-store/src/store.rs",
                "impl Table\n{\n    pub const fn All() -> &'static [Self]\n    {\n    }\n}\n",
            ),
            Source(
                "crates/spec/nomos-spec-store/src/governing.rs",
                "pub const GOVERNING_RECORD_IDS: &[&str] = &[];\n",
            ),
            Source("tests/contract/src/gates.rs", "pub const CORPUS_VARIABLES: &[&str] = &[];\n"),
        ]);

        let judged: Vec<&str> = findings
            .iter()
            .map(|finding| return finding.subject_name.as_str())
            .collect();

        assert_eq!(
            judged,
            vec!["CORPUS_VARIABLES", "GOVERNING_RECORD_IDS", "Table::All"],
            "all three instances OD-COMPLETENESS-001 analyses must be found"
        );
    }

    /// ---- the rule can say clean ----
    ///
    /// The control that stops the rule being a counter. A check that fires on everything
    /// is not a judgment, and a gate that can never be green is one everybody learns to
    /// ignore.
    #[test]
    fn Test_A_Universe_Whose_Mirror_Exists_Should_Produce_No_Finding()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
             pub const TABLES: &[&str] = &[];\n",
        );
        let checking = Source(
            "a_test.rs",
            "#[test]\nfn Test_Every_Table_Should_Be_Declared()\n{\n}\n",
        );
        let sources = vec![declaring.clone(), checking.clone()];

        let world = World_Over(&[
            (&declaring, &[]),
            (&checking, &["Test_Every_Table_Should_Be_Declared"]),
        ]);
        let mut reader = world.Reader();

        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// ---- the negative control this rule exists for ----
    ///
    /// A universe naming a check that does not exist reads as covered and checks nothing.
    /// It is the only outcome that blocks, and it must block: an admitted gap is honest,
    /// a false claim of coverage is not.
    #[test]
    fn Test_A_Mirror_That_Resolves_To_Nothing_Should_Block()
    {
        let findings = Findings_Over(&[Source(
            "a.rs",
            "/// Mirrored by `Test_Renamed_Away`.\npub const TABLES: &[&str] = &[];\n",
        )]);

        let finding = Only(&findings);

        assert_eq!(finding.gate, GateCategory::Blocking);
        assert!(
            finding.Can_Fail_A_Build(),
            "a claim of coverage that checks nothing must be able to stop a build"
        );
        assert!(
            finding.summary.contains("Test_Renamed_Away"),
            "the finding must name the check that resolved to nothing: {}",
            finding.summary
        );
    }

    /// The severity ordering, asserted directly, because it is the judgment call this
    /// rule makes and a later edit could quietly invert it.
    #[test]
    fn Test_A_False_Claim_Should_Outrank_An_Admitted_Gap()
    {
        let admitted = Findings_Over(&[Source("a.rs", "pub const TABLES: &[&str] = &[];\n")]);
        let false_claim = Findings_Over(&[Source(
            "a.rs",
            "/// Mirrored by `Test_Nowhere`.\npub const TABLES: &[&str] = &[];\n",
        )]);

        assert_eq!(Only(&admitted).gate, GateCategory::Advisory);
        assert_eq!(Only(&false_claim).gate, GateCategory::Blocking);
        assert!(
            Only(&false_claim).gate > Only(&admitted).gate,
            "a phantom mirror must outrank an admitted gap"
        );
        assert!(
            !Only(&admitted).Can_Fail_A_Build(),
            "an admitted gap is honest, and twelve of them exist; blocking on those \
             makes a gate that can never be green"
        );
    }

    /// A finding must carry its own provenance rather than leaving the reader to assume
    /// it. `Derived` is the honest class: computed from source by a deterministic rule.
    #[test]
    fn Test_A_Finding_Should_Report_How_It_Was_Come_By()
    {
        let findings = Findings_Over(&[Source("a.rs", "pub const T: &[&str] = &[];\n")]);
        let finding = Only(&findings);

        assert_eq!(finding.evidence, EvidenceClass::Derived);
        assert!(finding.Is_Mechanical());
        assert_eq!(finding.applicability, Applicability::Supported);
    }

    /// Identity is the name, not the path. A universe that moves file is the same
    /// universe, and a finding keyed on its location would close and reopen for free.
    #[test]
    fn Test_The_Same_Universe_In_Two_Places_Should_Keep_One_Identity()
    {
        let here = Findings_Over(&[Source("a.rs", "pub const T: &[&str] = &[];\n")]);
        let moved = Findings_Over(&[Source("b/c.rs", "pub const T: &[&str] = &[];\n")]);

        assert_eq!(Only(&here).subject, Only(&moved).subject);
        assert_ne!(Only(&here).locations, Only(&moved).locations);
    }

    /// An empty tree yields nothing, and that is exactly why the caller has to check for
    /// vacuity. Asserted here so the property is written down where the rule is, rather
    /// than being an unstated assumption the composition root happens to cover.
    #[test]
    fn Test_No_Sources_Should_Produce_No_Findings()
    {
        let world = World_Over(&[]);
        let mut reader = world.Reader();

        assert!(Check_Completeness_Mirrors(&[], &mut reader).is_empty());
    }

    /// The rule's own back door, under the new source of names. A check name written
    /// inside a fixture string is not a check, and resolving it would let a claim pass
    /// while nothing checks it.
    ///
    /// What keeps that true is no longer a parse performed here. It is the payload: a
    /// parser reports the items a file declares and a string literal is not one, so the
    /// name is not in the index. The fixture file's fact declares the function holding the
    /// fixture and nothing else, which is what `nomos-lang-rust` really writes. That the
    /// *provider* behaves this way and a line scanner does not is what
    /// [`crate::Syntax_Requirement`]'s floor is for, and it is asserted below.
    #[test]
    fn Test_A_Check_Named_Only_Inside_A_Fixture_Should_Not_Resolve()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_Only_In_A_Fixture`.\npub const T: &[&str] = &[];",
        );
        let fixture = Source(
            "b.rs",
            "fn Fixture() { let source = \"fn Test_Only_In_A_Fixture() {}\"; }",
        );
        let sources = vec![declaring.clone(), fixture.clone()];

        let world = World_Over(&[(&declaring, &[]), (&fixture, &["Fixture"])]);
        let mut reader = world.Reader();

        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        assert_eq!(Only(&findings).gate, GateCategory::Blocking);
    }

    /// A file that could not be read is reported, not skipped. A run that silently drops
    /// what it could not parse and reports clean is the shape this workspace keeps
    /// finding — and `Applicability` is the field that says so.
    #[test]
    fn Test_An_Unparseable_File_Should_Be_Reported_And_Not_Fail_The_Build()
    {
        let findings = Findings_Over(&[Source("broken.rs", "pub const ??? = ;")]);

        let unparseable = findings
            .iter()
            .find(|finding| return finding.summary.contains("could not be parsed"))
            .expect("the text side must report the file it could not read");

        assert_eq!(unparseable.applicability, Applicability::Unparseable);
        assert!(
            !unparseable.Can_Fail_A_Build(),
            "a rule that could not read its subject must not stop anybody"
        );
    }

    /// A mention is not a definition. Resolving against prose would let a comment naming
    /// a deleted test keep the claim alive, which is the defect one level up.
    #[test]
    fn Test_A_Test_Named_Only_In_Prose_Should_Not_Resolve()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_Deleted`.\npub const T: &[&str] = &[];\n",
        );
        let prose = Source("b.rs", "// see Test_Deleted for the comparison\n");
        let sources = vec![declaring.clone(), prose.clone()];

        let world = World_Over(&[(&declaring, &[]), (&prose, &[])]);
        let mut reader = world.Reader();

        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        assert_eq!(Only(&findings).gate, GateCategory::Blocking);
    }

    /// ---- the judgment reads a fact ----
    ///
    /// The load-bearing test for `OD-RULES-001`, and the one that would catch a fact path
    /// nothing consults. One tree, two runs, one difference: whether the store holds the
    /// fact for the file that defines the check. Withheld, the claim does not resolve;
    /// present, the rule reports nothing at all. The verdict turns on the fact and on
    /// nothing else — the text handed to the rule is byte-identical in both runs.
    #[test]
    fn Test_A_Mirror_Should_Resolve_Only_Through_A_Fact()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_Present`.\npub const T: &[&str] = &[];\n",
        );
        let checking = Source("b.rs", "#[test]\nfn Test_Present()\n{\n}\n");
        let sources = vec![declaring.clone(), checking.clone()];

        let with_fact = World_Over(&[(&declaring, &[]), (&checking, &["Test_Present"])]);
        let mut reader = with_fact.Reader();
        let resolved = Check_Completeness_Mirrors(&sources, &mut reader);

        assert!(
            resolved.is_empty(),
            "the fact for the defining file is present, so the claim resolves: {resolved:?}"
        );

        // The same two files, and the store is told about only one of them.
        let without_fact = World_Over(&[(&declaring, &[])]);
        let mut reader = without_fact.Reader();
        let unresolved = Check_Completeness_Mirrors(&sources, &mut reader);

        assert!(
            unresolved
                .iter()
                .any(|finding| return finding.subject_name == "b.rs"),
            "the subject whose fact was withheld must be named: {unresolved:?}"
        );
        assert!(
            unresolved
                .iter()
                .any(|finding| return finding.subject_name == "T"),
            "the claim must not resolve out of a text the rule can still see: {unresolved:?}"
        );
    }

    /// An empty store is not a clean tree.
    ///
    /// Every subject unread, so the rule must say it could not run rather than say
    /// nothing. Three properties together are what "could not run" means: the result is
    /// not empty, every finding names an unavailability rather than a phantom, and nothing
    /// in it can stop a build.
    #[test]
    fn Test_An_Empty_Store_Should_Not_Report_A_Clean_Tree()
    {
        let sources = vec![
            Source("a.rs", "/// Mirrored by `Test_Somewhere`.\npub const T: &[&str] = &[];\n"),
            Source("b.rs", "#[test]\nfn Test_Somewhere()\n{\n}\n"),
        ];

        let world = World::Offering(&[(PARSER, Parser_Guarantee())]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        assert!(
            !findings.is_empty(),
            "a run that read no fact at all must not render as a clean tree"
        );
        assert_eq!(
            findings
                .iter()
                .filter(|finding| {
                    return finding.applicability == Applicability::DependencyUnavailable;
                })
                .count(),
            3,
            "two unread subjects and one claim that could not be resolved: {findings:?}"
        );
        assert!(
            findings
                .iter()
                .all(|finding| return !finding.Can_Fail_A_Build()),
            "a rule that read nothing must not stop anybody: {findings:?}"
        );
        assert!(
            findings
                .iter()
                .all(|finding| return finding.gate != GateCategory::Blocking),
            "nothing may be reported as a phantom out of an index that was never built"
        );
    }

    /// A short index must not manufacture the one finding that blocks.
    ///
    /// One file's fact is missing and the universe claims a mirror defined in that file.
    /// The claim fails to resolve, and the rule may not call that a false claim of
    /// coverage: it cannot tell one from a name it did not get to look for.
    ///
    /// This is also the control that stops `OD-RULES-002` over-correcting into "every
    /// unresolved claim blocks". The scoping added there is a narrowing of *which* claims
    /// inherit the doubt and not a removal of the doubt, and this is the case it must
    /// still cover: the missing subject is exactly the one that would have resolved the
    /// name.
    #[test]
    fn Test_An_Incomplete_Index_Should_Not_Manufacture_A_Phantom()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_In_The_Unread_File`.\npub const T: &[&str] = &[];\n",
        );
        let unread = Source("b.rs", "#[test]\nfn Test_In_The_Unread_File()\n{\n}\n");
        let sources = vec![declaring.clone(), unread.clone()];

        let world = World_Over(&[(&declaring, &[])]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        let claim = findings
            .iter()
            .find(|finding| return finding.subject_name == "T")
            .expect("the universe is still judged");

        assert_eq!(claim.gate, GateCategory::Advisory);
        assert_eq!(claim.applicability, Applicability::DependencyUnavailable);
        assert!(
            !claim.Can_Fail_A_Build(),
            "a claim the rule could not check must not stop a build: {claim:?}"
        );
        assert!(
            claim.summary.contains("the check index is short b.rs"),
            "the finding must name the subject that could have resolved it: {}",
            claim.summary
        );
    }

    /// ---- OD-RULES-002: the shortfall is scoped to the claim it bears on ----
    ///
    /// The defect the record was opened against. One unreadable subject anywhere silenced
    /// every phantom in the tree, and over this workspace there is always one:
    /// `tests/corpus/analysis/gamma/broken.rs` is a fixture the parser is supposed to
    /// refuse. So the guard reported and never judged.
    ///
    /// Here the phantom is claimed in a file whose fact was read, and the unreadable file
    /// is present in the same run and does not spell the claimed name. Those two facts have
    /// nothing to do with each other, and the judgment must be made.
    #[test]
    fn Test_A_Phantom_In_A_Read_Subject_Should_Block_Though_Another_Subject_Was_Unread()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
        );
        // The same shape as the real fixture: text no parser accepts, and no fact.
        let broken = Source("broken.rs", "pub const ??? = ;\n");
        let sources = vec![declaring.clone(), broken.clone()];

        let world = World_Over(&[(&declaring, &[])]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        let claim = findings
            .iter()
            .find(|finding| return finding.subject_name == "T")
            .expect("the universe is judged");

        assert_eq!(claim.gate, GateCategory::Blocking);
        assert_eq!(claim.applicability, Applicability::Supported);
        assert!(
            claim.Can_Fail_A_Build(),
            "a phantom claimed in a file whose facts were read must stop a build whatever \
             some other subject did: {claim:?}"
        );

        // The caveat travels with the judgment rather than replacing it. A reader who wants
        // to know what the run did not see is owed that on the finding, not instead of it.
        assert!(
            claim.summary.contains("short 1 subject(s)")
                && claim.summary.contains("Test_Renamed_Away"),
            "the blocking finding must carry the shortfall it ruled out: {}",
            claim.summary
        );

        // And the unreadable subject is still reported, twice, exactly as before.
        assert_eq!(
            findings
                .iter()
                .filter(|finding| return finding.subject_name == "broken.rs")
                .count(),
            2,
            "both readings of the unreadable file must still be reported: {findings:?}"
        );
    }

    /// The second control, and the one that stops the correction going too far.
    ///
    /// A claim sited in a subject whose own fact could not be read, naming a check that
    /// subject itself declares. Nothing resolves it and it is still not a phantom: the file
    /// that would have resolved it is the file the index is short of. A universe and the
    /// test that mirrors it living in one file is the ordinary shape in this workspace, so
    /// this is not a corner.
    #[test]
    fn Test_A_Claim_In_An_Unread_Subject_Should_Not_Be_A_Phantom()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_Right_Here`.\n\
             pub const T: &[&str] = &[];\n\
             #[cfg(test)]\n\
             mod tests\n\
             {\n\
             \x20   #[test]\n\
             \x20   fn Test_Right_Here()\n\
             \x20   {\n\
             \x20   }\n\
             }\n",
        );
        let other = Source("b.rs", "#[test]\nfn Test_Something_Else()\n{\n}\n");
        let sources = vec![declaring.clone(), other.clone()];

        // Every fact but the declaring file's own, so the index is real and short of it.
        let world = World_Over(&[(&other, &["Test_Something_Else"])]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        let claim = findings
            .iter()
            .find(|finding| return finding.subject_name == "T")
            .expect("the universe is discovered from a text syn can still read");

        assert_eq!(claim.gate, GateCategory::Advisory);
        assert_eq!(claim.applicability, Applicability::DependencyUnavailable);
        assert!(
            !claim.Can_Fail_A_Build(),
            "the file that would have resolved this claim is the one the index is short \
             of: {claim:?}"
        );
        assert!(
            claim.summary.contains("the check index is short a.rs"),
            "{}",
            claim.summary
        );
    }

    /// The unsound half of the scoping, in the direction it is allowed to be unsound.
    ///
    /// `Could_Have_Declared` is a substring test over a text no parser read, so a name
    /// spelled only in a comment counts as "could have declared it". That is a false
    /// positive and it *withholds* a block, which is the safe direction and the reason an
    /// unsound signal is admissible here at all: `Check_Index_Of` may not use text as a
    /// source of names because an unsound positive there resolves a claim and silences the
    /// rule, and this one can only ever add doubt.
    #[test]
    fn Test_A_Name_Spelled_In_A_Comment_Should_Still_Withhold_The_Block()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_Only_Mentioned`.\npub const T: &[&str] = &[];\n",
        );
        let broken = Source(
            "broken.rs",
            "// Test_Only_Mentioned used to live here.\npub const ??? = ;\n",
        );
        let sources = vec![declaring.clone(), broken.clone()];

        let world = World_Over(&[(&declaring, &[])]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        let claim = findings
            .iter()
            .find(|finding| return finding.subject_name == "T")
            .expect("the universe is judged");

        assert_eq!(
            claim.gate,
            GateCategory::Advisory,
            "an unread file that spells the name is doubt, however it spells it: {claim:?}"
        );
        assert!(!claim.Can_Fail_A_Build());
    }

    /// Two claims, one run, one unreadable subject — and they are judged differently.
    ///
    /// The sharpest form of what `OD-RULES-002` changed. Under one flag over the whole run
    /// these two claims were indistinguishable and both were downgraded. The shortfall is a
    /// property of the subjects each judgment rests on, so the run now says two different
    /// things in the same breath.
    #[test]
    fn Test_Two_Claims_Under_One_Shortfall_Should_Be_Judged_Separately()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_In_The_Broken_File`.\n\
             pub const WITHHELD: &[&str] = &[];\n\
             /// Mirrored by `Test_Nowhere_At_All`.\n\
             pub const PHANTOM: &[&str] = &[];\n",
        );
        let broken = Source("broken.rs", "fn Test_In_The_Broken_File( ??? = ;\n");
        let sources = vec![declaring.clone(), broken.clone()];

        let world = World_Over(&[(&declaring, &[])]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        let judged: Vec<(&str, GateCategory)> = findings
            .iter()
            .filter(|finding| return finding.subject_name.starts_with(char::is_uppercase))
            .map(|finding| return (finding.subject_name.as_str(), finding.gate))
            .collect();

        assert_eq!(
            judged,
            vec![
                ("PHANTOM", GateCategory::Blocking),
                ("WITHHELD", GateCategory::Advisory),
            ],
            "one shortfall bears on one of these two claims and not the other: {findings:?}"
        );
    }

    /// And the shortfall is still whole-run when there is no index at all.
    ///
    /// `MissingCapability` is not scoped by what a subject spells, and the reason is not a
    /// carve-out: with no admitted provider there is no index, so no name can be shown
    /// absent from it. Without this arm a run against a composition that offers the rule
    /// nothing would block on every claim in the tree — a verdict reached by a rule that
    /// never got an answer from anybody, which is what `OD-RULES-001` forbids.
    #[test]
    fn Test_With_No_Provider_Admitted_No_Claim_Should_Be_A_Phantom()
    {
        let source = Source(
            "a.rs",
            "/// Mirrored by `Test_Nowhere_In_This_Tree`.\npub const T: &[&str] = &[];\n",
        );

        let world = World::Offering(&[]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&[source], &mut reader);

        let claim = findings
            .iter()
            .find(|finding| return finding.subject_name == "T")
            .expect("the universe is judged");

        assert_eq!(claim.applicability, Applicability::MissingCapability);
        assert_eq!(claim.gate, GateCategory::Advisory);
        assert!(!claim.Can_Fail_A_Build());
        assert!(
            claim.summary.contains("no check index for this name to be absent from"),
            "{}",
            claim.summary
        );
    }

    /// The control that stops the tests above from being satisfied by a rule that never
    /// blocks. Every subject read, one claim that resolves to nothing, and it blocks.
    #[test]
    fn Test_A_Phantom_Should_Still_Block_When_Every_Subject_Was_Read()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
        );
        let other = Source("b.rs", "#[test]\nfn Test_Something_Else()\n{\n}\n");
        let sources = vec![declaring.clone(), other.clone()];

        let world = World_Over(&[(&declaring, &[]), (&other, &["Test_Something_Else"])]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        let claim = Only(&findings);

        assert_eq!(claim.gate, GateCategory::Blocking);
        assert_eq!(claim.applicability, Applicability::Supported);
        assert!(claim.Can_Fail_A_Build());
    }

    /// An admitted gap stays advisory and stays *evaluated* even when the index is short.
    ///
    /// `D-134`'s decision 4 is that a false claim outranks an admitted gap, and the
    /// amendment must not weaken the second half of it by letting an incomplete index
    /// downgrade a universe that never claimed anything. Nothing was resolved for it, so
    /// nothing could have been missed.
    #[test]
    fn Test_An_Admitted_Gap_Should_Not_Inherit_The_Indexs_Doubt()
    {
        let declaring = Source("a.rs", "pub const T: &[&str] = &[];\n");
        let unread = Source("b.rs", "#[test]\nfn Test_Whatever()\n{\n}\n");
        let sources = vec![declaring.clone(), unread.clone()];

        let world = World_Over(&[(&declaring, &[])]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&sources, &mut reader);

        let gap = findings
            .iter()
            .find(|finding| return finding.subject_name == "T")
            .expect("the universe is judged");

        assert_eq!(gap.gate, GateCategory::Advisory);
        assert_eq!(gap.applicability, Applicability::Supported);
        assert!(gap.summary.contains("declares no mirror"), "{}", gap.summary);
    }

    /// Nothing offers what the rule needs, which is a different sentence from "the store
    /// had nothing". The remedy is registering a provider rather than running one, and
    /// `Applicability` is where the difference is carried.
    #[test]
    fn Test_A_Composition_With_No_Provider_Should_Report_A_Missing_Capability()
    {
        let source = Source("a.rs", "pub const T: &[&str] = &[];\n");

        let world = World::Offering(&[]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&[source], &mut reader);

        assert!(
            findings
                .iter()
                .any(|finding| return finding.applicability == Applicability::MissingCapability),
            "{findings:?}"
        );
    }

    /// The floor buys something, and this is what.
    ///
    /// A composition that registered only a line scanner cannot serve this rule: the offer
    /// is below the floor, the registry refuses it, and the rule reports that it could not
    /// run rather than resolving claims against names a scanner found inside comments.
    /// Asserted against a guarantee written out here rather than against
    /// `nomos-lang-rust-scan`, which this crate must not name.
    #[test]
    fn Test_The_Scanners_Guarantee_Should_Not_Satisfy_This_Rules_Floor()
    {
        let floor = Syntax_Requirement().minimum;

        assert!(
            !Scanner_Guarantee().Satisfies(&floor),
            "an unsound approximation must not be admissible as a source of check names"
        );
        assert!(
            Parser_Guarantee().Satisfies(&floor),
            "a floor no provider can meet is a declared need with nothing behind it"
        );

        let source = Source(
            "a.rs",
            "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
        );
        let world = World::Offering(&[(SCANNER, Scanner_Guarantee())]);
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&[source], &mut reader);

        let claim = findings
            .iter()
            .find(|finding| return finding.subject_name == "T")
            .expect("the universe is still discovered from its text");

        assert_eq!(claim.applicability, Applicability::MissingCapability);
        assert!(
            !claim.Can_Fail_A_Build(),
            "with only a scanner admitted the rule could not run, so it may not block"
        );
    }

    /// A fact stamped with a schema this build does not read is not an empty file.
    ///
    /// The payload's shape is versioned separately from the contract, so a future provider
    /// can answer the same question in a shape this reader has never seen. Decoding it
    /// anyway would build a check index out of a guess.
    #[test]
    fn Test_A_Payload_Under_Another_Schema_Should_Not_Be_Decoded()
    {
        let source = Source("a.rs", "pub const T: &[&str] = &[];\n");

        let world = World::Offering(&[(PARSER, Parser_Guarantee())]).Materializing(
            &source,
            "nomos.syntax.items.v9",
            Payload(&["Test_From_The_Future"]),
        );
        let mut reader = world.Reader();
        let findings = Check_Completeness_Mirrors(&[source], &mut reader);

        let unread = findings
            .iter()
            .find(|finding| return finding.subject_name == "a.rs")
            .expect("the subject must be reported unread");

        assert_eq!(unread.applicability, Applicability::Unparseable);
        assert!(unread.summary.contains("v9"), "{}", unread.summary);
    }
}
