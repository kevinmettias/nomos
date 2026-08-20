//! Band 3 — rules that judge source.
//!
//! The first thing in this workspace that judges code rather than judging the
//! workspace's own paperwork. `P10-FIRST-CHECK` opened for that reason: four types in
//! `nomos-contracts` described enforcement and nothing implemented them, and two
//! consecutive batches of work had produced audit rather than capability.
//!
//! # A rule takes its subject as an argument
//!
//! Nothing in this crate opens a file, walks a directory, or knows where the workspace
//! is. A rule takes [`SourceFile`]s and a [`nomos_analysis::FactReader`] and returns
//! findings, and the caller supplies both — the binary composes them from the real tree,
//! and a test composes them from three files it wrote by hand.
//!
//! That is not a style preference. `P10-FIRST-CHECK` requires the three instances
//! `OD-COMPLETENESS-001` analyses to fail this rule *as originally written*, and none of
//! the three can be replayed from git: each was repaired at the site. The only way to
//! judge code that no longer exists is for the rule to accept its subject as an
//! argument. A rule that reads the filesystem can only ever be tested against the tree
//! it is standing in.
//!
//! `D-134` recorded that argument and drew a wider conclusion from it — that the subject
//! must be *text*. It does not follow, and `OD-RULES-001` withdraws it: a reader handed
//! in as a parameter is an argument in exactly the sense a `&[SourceFile]` is, and
//! the replay property is met identically. `D-134` is amended at version 2 rather than
//! superseded; six of its seven decisions stand untouched.
//!
//! # Where this rule gets each half of what it needs
//!
//! Two halves, and they fall on opposite sides of what the agreed payload of
//! `nomos.cap.syntax.items` can carry.
//!
//! *Resolving a claimed mirror* against the checks that really exist is read from facts.
//! [`Syntax_Requirement`] is the floor this crate states for itself, and it is the rule's
//! and not the caller's: a composition root cannot lower it, so a line scanner that
//! reports a `fn Test_Renamed_Away` sitting inside a block comment cannot be substituted
//! for a parser and quietly resolve a claim that nothing checks.
//!
//! *Discovering which declarations are universes* still reads the text it is handed,
//! because the payload carries neither doc comments nor declared types and no version of
//! it exists that would. That is a measurement rather than a preference, and it names its
//! own end condition — see `universe.rs` and `OD-RULES-001`.
//!
//! # What is here
//!
//! Four rules. [`Check_Completeness_Mirrors`] was chosen first because it is the only
//! rule in this tree with three recorded historical instances to test a judgment against —
//! `P10-FIRST-CHECK` shipped with exactly this one and no more, because a single check
//! that is honest end to end is worth more than three that are nearly wired.
//! [`Check_Naming_Convention`] is the second, added once a real second rule was needed to
//! test `OD-PACKAGE-008`'s question — whether a future `RulePackage` manifest's field
//! boundaries generalize past a population of one — against something other than the
//! first rule's own shape. It judges a different kind of claim (a lexical convention
//! stated once in prose, not a per-subject doc comment) for exactly that reason.
//! [`Check_Dependency_Direction`] is the third, designed by `OD-RULES-003`: a declared
//! architecture is data, the observed dependency graph is a fact a capability provider
//! establishes, and this rule composes the two into findings — the same property
//! `tests/contract/tests/boundaries/graph.rs` already enforces for this repository by
//! hand. `P13-DEPENDENCY-EDGES-2` landed the rule and `P13-DEPENDENCY-WIRE-1` composed it
//! into `nomos-check-orchestration`'s real `Run`. [`Check_Unread_Reaches_A_Finding`] is
//! the fourth, the candidate `OD-RULES-008` named and `P13-CONTROLFLOW-REACHABILITY-
//! CAPABILITY` built: a control-flow edge inside one function body either reaches a
//! `Finding` after a fact-read failure or it does not, the first judgment in this crate
//! that is not a flat per-subject decode-and-compare. Neither it nor its provider is
//! composed into `nomos-check-orchestration::Run` yet — a follow-on item, the same split
//! `Check_Dependency_Direction` used.
//!
//! [`RuleRegistry`] is a fourth thing, deliberately not a rule: `OD-RULES-004` extracted a
//! registration contract ahead of a second rule, so a rule package can be designed and
//! built against a stated shape rather than by copying this crate's own hand-written
//! composition. It is additive and unconsulted — `Run()` still calls each rule directly,
//! unconditionally, exactly as `OD-HOST-004` decided, and nothing here changes what runs
//! on any given `nomos check`.

#![forbid(unsafe_code)]

mod declared_universe;
mod dependency;
mod facts;
mod mirror;
mod naming;
mod reachability;
mod reading;
mod registry;
mod universe;

use nomos_capability::Requirement;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SubjectId};

pub use mirror::{Check_Completeness_Mirrors, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION};
pub use declared_universe::DeclaredUniverse;
pub use dependency::{Check_Dependency_Direction, DEPENDENCY_CONTRACT_RECORD, DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION};
pub use naming::{Check_Naming_Convention, NAMING_CONVENTION};
pub use reachability::{
    Check_Unread_Reaches_A_Finding, UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
};
pub use reading::Reading;
pub use registry::{RuleOffer, RuleRegistry, RuleRegistryError};
pub use universe::{UniverseKind, Universes_In};

/// What this crate needs from a syntax provider before it will believe an answer.
///
/// A floor, not a preference, and stated by the rule rather than by whoever runs it.
///
/// # Why the floor is here and not at the call site
///
/// Because the rule is the party that knows what an approximation would cost it. A
/// composition root that could lower this would be able to feed the resolver a line
/// scanner, and `nomos-lang-rust-scan`'s own
/// `Test_The_Declared_Unsoundness_Should_Be_Demonstrable` establishes exactly what that
/// buys: it reports declarations written inside block comments. A `fn Test_Renamed_Away`
/// in a comment or a string literal would then resolve a mirror claim that nothing checks
/// — the defect this rule exists to find, arriving through the rule's own resolver.
/// [`nomos_contracts::Guarantee::Satisfies`] refuses that offer against this floor and
/// `Registry::Resolve` reports `Unmet::BelowRequirement`.
///
/// # Why each axis is what it is
///
/// [`FactVariant::Syntactic`] because a check name is what a file says on its face;
/// nothing here resolves a name or follows a `use`.
///
/// Soundness [`Assurance::Sound`] because every name that resolves a claim of coverage
/// must really be in the token stream. This is the axis that separates the two providers
/// and the only one this rule cannot compromise on.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound`. A parser cannot bound
/// what a macro hid, so no provider of this capability can honestly claim complete — and
/// a floor no provider can meet is not caution, it is a declared need with nothing behind
/// it. What follows from an unknown-complete index is handled where it matters: a check
/// name the index is missing produces a phantom finding, and `mirror.rs` refuses to raise
/// one while any subject went unread.
///
/// [`IncrementalGranularity::File`] because a check name belongs to the file that declares
/// it, and nothing coarser would let one edited file be re-read on its own.
///
/// There is deliberately no `Preferring`. Naming a provider would be the rule deciding
/// what the registry exists to decide.
#[must_use]
pub(crate) fn Syntax_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );

    return Requirement::New(
        nomos_cap_syntax::Capability(),
        nomos_cap_syntax::CONTRACT_VERSION,
        guarantee,
    );
}

/// One file of source, as the caller found it.
///
/// `path` is repo-relative with forward slashes, and it is reporting only. Nothing in
/// this crate keys anything on it: `identity.rs` states the rule for the whole system,
/// and a finding identified by its path is a finding that closes and reopens every time
/// somebody moves a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile
{
    /// Repo-relative, forward slashes. For reporting.
    pub path: String,
    /// The subject the composition root filed this file's facts under.
    ///
    /// Carried rather than derived, and that is the whole of why it is a field. A
    /// [`SubjectId`] is a digest of a *normalized* path, and normalization is an
    /// addressing convention the root owns — `tests/integration/src/corpus.rs` folds
    /// case and drops `.` segments and says why. A rule that computed its own would be a
    /// second answer to that convention, and the two would disagree silently: every
    /// `Require` would miss, every mirror would fail to resolve, and the run would report
    /// a workspace full of unavailable facts rather than a mistake in one function.
    /// Carrying it makes rule and root agree by construction.
    ///
    /// The *inputs* digest is not carried and is recomputed in the rule from `text`. That
    /// is the same deliberate choice `tests/integration/src/slice.rs` documents: if the two
    /// disagree the read misses loudly, and a shared helper would make that whole class of
    /// mismatch untestable.
    pub subject: SubjectId,
    /// The file's full text.
    pub text: String,
}

impl SourceFile
{
    /// Builds one, for callers that have a path, a subject and the text.
    #[must_use]
    pub fn New(path: impl Into<String>, subject: SubjectId, text: impl Into<String>) -> Self
    {
        return Self {
            path: path.into(),
            subject,
            text: text.into(),
        };
    }
}
