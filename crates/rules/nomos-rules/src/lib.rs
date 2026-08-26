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
//! Eight rules. [`Check_Completeness_Mirrors`] was chosen first because it is the only
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
//! that is not a flat per-subject decode-and-compare. `P13-CONTROLFLOW-REACHABILITY-WIRE`
//! composed it into `nomos-check-orchestration::Run`, the same way `P13-DEPENDENCY-WIRE-1`
//! composed `Check_Dependency_Direction`.
//!
//! [`RuleRegistry`] is a fourth thing, deliberately not a rule: `OD-RULES-004` extracted a
//! registration contract ahead of a second rule, so a rule package can be designed and
//! built against a stated shape rather than by copying this crate's own hand-written
//! composition. It is additive and unconsulted for selection — `Run()` originally called
//! each rule directly, unconditionally, exactly as `OD-HOST-004` decided; `OD-GATE-017`
//! superseded that with a real per-call `selected: &[RuleId]` gate, not `RuleRegistry`,
//! so [`RuleRegistry`] itself still changes nothing about what runs on any given
//! `nomos check`.
//!
//! [`Check_Declared_Role_Matches_Surface`] is a fifth rule, additive and unwired into
//! `nomos-check-orchestration::Run` the same way [`RuleRegistry`] was before a second rule
//! existed to check its shape against. It is the first rule in this crate to always resolve
//! [`nomos_contracts::Applicability::AgentRequired`] — `CHK-003`'s seventh reporting
//! category, real since `OD-CONTRACTS-002` but produced by no rule until this one — because
//! whether a crate's declared role and its actual public surface agree is a semantic
//! judgment no mechanical provider can make, grounded against real, external
//! architecture-standards precedent (`role_surface.rs`'s own module doc names it) rather
//! than invented. It takes plain data instead of a [`nomos_analysis::FactReader`], and its
//! own module doc explains why.
//!
//! [`Check_Lint_Diagnostics`] is a sixth rule, `OD-RULES-010`'s own first `ToolProvider`
//! increment: `nomos.cap.lint.diagnostics` facts, materialized by `nomos-lang-rust-clippy`
//! from a real `cargo clippy` run, relayed 1:1 as `Finding`s rather than judged a second
//! time — the first rule in this crate whose whole judgment is "the tool already decided,"
//! stated as its own `Requirement` the same way every other rule states its own floor.
//! Composed into `nomos-check-orchestration::Run`, behind `OD-GATE-017`'s own per-call rule
//! subset, the same `Wants(selected, ...)` gate `DEPENDENCY_DIRECTION` already sits behind.
//!
//! [`Check_Dependency_Policy`] is a seventh rule, `OD-RULES-010`'s second real
//! `ToolProvider` instance: `nomos.cap.dependency.policy` facts, materialized by
//! `nomos-lang-rust-deny` from a real `cargo deny check bans licenses sources` run,
//! relayed 1:1 the identical way [`Check_Lint_Diagnostics`] already relays `cargo clippy`'s
//! own verdict — minus even that rule's own per-member loop, since this capability's one
//! real provider materializes exactly one fact for the whole workspace.
//!
//! [`Check_Cross_Language_Correspondence`] is an eighth rule, `OD-CAPABILITY-010`'s own
//! work: the first rule in this crate that reads one capability twice for one judgment,
//! over a subject *pair* a doc-comment-declared correspondence names rather than a subject
//! the walk handed it directly. No new capability — both sides are already `nomos.cap.
//! syntax.items` facts — and no new materialization, since every source's syntax fact is
//! already written before any rule runs.

#![forbid(unsafe_code)]

mod crosslang;
mod declared_universe;
mod dependency;
mod facts;
mod lint;
mod mirror;
mod naming;
mod policy;
mod reachability;
mod reading;
mod registry;
mod role_surface;
mod universe;

use nomos_capability::Requirement;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId, SubjectId};

pub use mirror::{Check_Completeness_Mirrors, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION};
pub use declared_universe::DeclaredUniverse;
pub use dependency::{Check_Dependency_Direction, DEPENDENCY_CONTRACT_RECORD, DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION};
pub use lint::{Check_Lint_Diagnostics, LINT_DIAGNOSTICS};
pub use policy::{Check_Dependency_Policy, DEPENDENCY_POLICY};
pub use crosslang::{Check_Cross_Language_Correspondence, CROSS_LANGUAGE_CORRESPONDENCE};
pub use naming::{Check_Naming_Convention, NAMING_CONVENTION};
pub use reachability::{
    Check_Unread_Reaches_A_Finding, UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
};
pub use reading::Reading;
pub use registry::{RuleOffer, RuleRegistry, RuleRegistryError};
pub use role_surface::{Check_Declared_Role_Matches_Surface, RoleSurfacePair, DECLARED_ROLE_MATCHES_SURFACE};
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

/// [`Syntax_Requirement`], narrowed to `preferred` when the caller names one —
/// `OD-CAPABILITY-009`'s decided fix for a capability whose real offers partition by
/// subject rather than compete over one.
///
/// Takes the preference as data rather than computing it from a path: this crate never
/// depends on a language-provider crate, and recognizing which language a path belongs to
/// is exactly that kind of dependency. The composition root already depends on every
/// registered syntax provider by name — it is the one place allowed to compute
/// `Recognition::Of_Path`, once, and carry the result here as
/// [`SourceFile::preferred_syntax_provider`], the same carried-rather-than-derived
/// convention that field's own sibling `subject` already documents. `None` carries no
/// preference and falls through to the floor above, unpreferenced — the case of a path
/// neither registered provider recognizes, which `OD-CAPABILITY-009` names explicitly
/// rather than leaves implicit. That fallthrough is still safe: nothing materializes a
/// fact under either provider's identity for a path neither recognizes, so an unrecognized
/// path still surfaces as an honestly unread subject rather than a wrongly-addressed one.
#[must_use]
pub(crate) fn Syntax_Requirement_For(preferred: Option<ProviderId>) -> Requirement
{
    let need = Syntax_Requirement();

    return match preferred
    {
        Some(provider) => need.Preferring(provider),
        None => need,
    };
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
    /// Which `nomos.cap.syntax.items` provider identity to narrow toward when this file's
    /// syntax fact is required, if any — `OD-CAPABILITY-009`'s fix for a capability whose
    /// real offers partition by subject rather than compete over one.
    ///
    /// Carried rather than derived, for the identical reason [`SourceFile::subject`] is: a
    /// capability with more than one registered offer over disjoint subjects (today,
    /// `nomos-lang-rust` over `.rs` and `nomos-lang-go` over `.go`) needs its read side and
    /// its write side to agree on which provider answers for one file, and computing that
    /// twice independently is how the two sides drift. The composition root recognizes
    /// `path` against every provider it registers and sets this once, before any rule ever
    /// sees the file — this crate itself never depends on a language-provider crate to
    /// compute it. `None` means either no registered provider recognizes this path, or the
    /// caller named none; [`Syntax_Requirement_For`] treats both the same way, falling
    /// through to the subject-agnostic floor.
    pub preferred_syntax_provider: Option<ProviderId>,
}

impl SourceFile
{
    /// Builds one, for callers that have a path, a subject and the text.
    ///
    /// `preferred_syntax_provider` starts `None` — a caller that has already resolved one
    /// sets the field directly, since every field here is public for exactly that reason.
    #[must_use]
    pub fn New(path: impl Into<String>, subject: SubjectId, text: impl Into<String>) -> Self
    {
        return Self {
            path: path.into(),
            subject,
            text: text.into(),
            preferred_syntax_provider: None,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Named_Preference_Should_Narrow_The_Floor()
    {
        let preferred = ProviderId::New("nomos.test.provider");

        let need = Syntax_Requirement_For(Some(preferred.clone()));

        assert_eq!(need.preferred, Some(preferred));
        assert_eq!(need.minimum, Syntax_Requirement().minimum, "the floor itself is untouched");
    }

    /// The case `OD-CAPABILITY-009` names explicitly: no preference is named, and the
    /// floor falls through to the registry's own, subject-agnostic ranking — the same as
    /// before this fix existed.
    #[test]
    fn Test_No_Preference_Should_Carry_The_Bare_Floor_Through_Unchanged()
    {
        let need = Syntax_Requirement_For(None);

        assert_eq!(need, Syntax_Requirement());
    }
}
