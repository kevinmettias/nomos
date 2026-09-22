//! The analysis providers a run materializes through, as values rather than as names.
//!
//! Every materialization step in this crate used to call a provider crate by its real path:
//! `nomos_lang_rust::Materialize_Syntax_Fact`, `nomos_repo_policy::naming::
//! Materialize_Workspace`, `nomos_lang_rust_clippy::Materialize_Workspace` and some thirty
//! more across sixteen files. An external architecture review called the result a
//! composition root disguised as an application service, and `OD-ROADMAP-005` decision item
//! 1 authorizes the move: the service stops naming the capability, language and repository
//! provider crates and receives them instead.
//!
//! # Why a table of ports and not one trait
//!
//! `OD-CAPABILITY-008` measured the provider convention field by field across every real
//! provider and found three of its four parts genuinely agree -- `PROVIDER`,
//! `Declared_Guarantee()` and the four-field `FactContext` -- while `Materialize` does not,
//! because its divergence tracks a real difference in what each capability is *of*: a
//! per-file parse against a whole-workspace discovery, one fact against many, a filesystem
//! read against a subprocess launch. That record refuses a single trait spanning all of
//! them, on the grounds that a signature general enough to admit both would be a new shape
//! invented to paper over two capability kinds that are not the same kind.
//!
//! So this module declares **one port per granularity that really exists, and no fewer**.
//! Each provider keeps exactly the signature it has today; what the ports remove is the
//! service's knowledge of *whose* signature it is. The three parts `OD-CAPABILITY-008`
//! measured as agreeing are the three fields [`SyntaxProvider`] carries beside its
//! materialization, which is the narrow trait that record licensed, built as data rather
//! than as a trait because there is one consumer and it needs a list it can iterate.
//!
//! The fourth agreeing part moves with them. Adapting [`nomos_analysis::Context`] into a
//! provider's own `FactContext` is four field assignments, written out twelve times across
//! this crate today -- one private `*_Production` function per materialization file. It is
//! the one part of the convention that never differs, so every port here carries a
//! `&Context` and the adaptation happens once per provider at the composition site that
//! already knows the provider by name.
//!
//! # What a port is not
//!
//! A port is a declared constant. `OD-ROADMAP-003`'s surviving constraint is that a new
//! provider composition or materialization section stays a declared row and never becomes a
//! condition consulting store state, cost or prior materialization; nothing in this module
//! reads any of the three, and the table is a struct literal built once per run. Demand is
//! still decided where it was, by `crate::run_context::capabilities::Demanded_Families`
//! reading `nomos_rules::DESCRIPTORS`, and currency is still decided where it was, by
//! `crate::facts::currency`, after a provider has already answered.

use nomos_analysis::{Context, MaterializedFact};
use nomos_cap_syntax::Language;
use nomos_contracts::{Guarantee, ProviderId, SubjectId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use std::path::Path;

/// One `nomos.cap.syntax.items` provider as this run holds it: its identity, the language it
/// reads, the guarantee it declares, whether it recognizes a path, and how it turns one
/// source's text into one fact.
///
/// The four fields beside `materialize` are exactly what `OD-CAPABILITY-008` measured as
/// genuinely agreeing across every provider in this workspace, plus the recognition question
/// `OD-CAPABILITY-009` decided the caller must answer rather than `Registry::Resolve`: by the
/// time `Resolve` is reached the subject is the opaque digest [`SubjectId`] carries, so a
/// capability whose offers partition by subject is narrowed here instead.
///
/// `materialize` is the per-file granularity, and it is per-file for both providers that
/// have it -- `OD-CAPABILITY-008` found the Rust and Go syntax providers agree on this
/// signature, and it is the `Materialize` shape whose *other* instances it found principled
/// to leave alone. `None` means the provider read the source and refused it, which is not a
/// finding: the run counts the subject as uncovered and the rule's own unread-subject
/// finding names it.
pub struct SyntaxProvider
{
    pub provider: ProviderId,
    pub language: Language,
    pub guarantee: Guarantee,
    pub recognizes: fn(&str) -> bool,
    pub materialize: fn(SubjectId, &str, &Context) -> Option<Box<MaterializedFact>>,
}

/// One fact a whole-workspace provider produced, addressed to the subject it belongs to.
///
/// The neutral carrier for what `nomos_lang_rust_cargo::PackageFact`,
/// `nomos_lang_rust_clippy::DiagnosticsFact` and `nomos_lang_rust_deny::PolicyFact` each
/// name in their own vocabulary. `path` is repository-relative with forward slashes, the
/// same path `subject` was addressed from; a capability whose ceiling is the whole workspace
/// rather than one member carries the literal the service already gave it.
pub struct SubjectFact
{
    pub path: String,
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}

/// A per-subject provider with no port of its own: reads one source's text and answers one
/// fact, or refuses.
///
/// The `nomos.cap.controlflow.reachability` granularity. Identical in shape to
/// [`SyntaxProvider::materialize`] and deliberately not folded into it: reachability has one
/// offer, no recognition question and no second provider to rank against, so the four
/// identity fields beside a syntax provider would be four values with nothing to
/// distinguish.
pub type SubjectFactProvider = fn(SubjectId, &str, &Context) -> Option<Box<MaterializedFact>>;

/// A whole-workspace provider read through a filesystem port, answering exactly one fact.
///
/// The granularity `OD-CAPABILITY-008` measured across six real bodies and licensed a trait
/// over on that evidence -- `Materialize_Workspace(root, FactContext, port) ->
/// Result<PolicyFact, Error>` -- now eight, with `nomos.cap.architecture.declaration` and
/// `nomos.cap.requirement.trace` joining the six repository-declared policy families.
///
/// `None` is every reason the provider did not answer, collapsed, because the service does
/// nothing with the distinction: `OD-CAPABILITY-004` already decided an absent optional
/// capability is silently "no override" from whichever rule reads it, so this family's
/// refusal has never produced a finding and has never been inspected.
pub type WorkspacePolicyProvider<Fs> = fn(&Path, &Context, &Fs) -> Option<MaterializedFact>;

/// A whole-workspace provider read through a process launcher, answering exactly one fact.
///
/// `nomos_lang_rust_deny`'s granularity, and the reason it is not
/// [`WorkspacePolicyProvider`] with a different port: a refusal here *is* reported, as the
/// `ProviderUnavailable` finding the service builds, so the error has to survive the call.
/// It survives as its rendered text rather than as its type because rendering is the only
/// thing this crate has ever done with it -- `format!("{error}")` into one finding summary,
/// at each of the three sites that have one.
pub type WorkspaceFactProvider<Launcher, Env> = fn(&Path, &Context, &Launcher, &Env) -> Result<SubjectFact, String>;

/// A whole-workspace provider read through a process launcher, answering one fact per
/// workspace member.
///
/// `nomos_lang_rust_cargo`'s and `nomos_lang_rust_clippy`'s granularity -- the
/// `Result<Vec<Fact>, Error>` shape `OD-CAPABILITY-008` named as a *different capability
/// kind* from the one-fact family, not a drifted copy of it, and which it therefore declined
/// to unify. It is a separate port here for that reason and not because the two happen to
/// differ today.
pub type SubjectFactsProvider<Launcher, Env> = fn(&Path, &Context, &Launcher, &Env) -> Result<Vec<SubjectFact>, String>;

/// Every analysis provider one run materializes through, as one value.
///
/// One field per capability this crate's composition declares an offer for, at the port its
/// granularity requires. A capability declared with no materialization step -- `nomos.cap.
/// review.finding`, whose provider answers about an already-identified external review
/// comment nothing in a walk names -- has no field here, the same way it has no
/// materialization function.
pub struct ComposedProviders<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    /// Every `nomos.cap.syntax.items` offer that can answer for a walked path, in the order
    /// [`Recognized_Syntax_Provider`] consults them. Order is the tie-break a list has and a
    /// registry does not: `OD-CAPABILITY-009` decided the caller narrows a subject-partitioned
    /// capability rather than `Registry::Resolve` ranking it, and first-match is how that
    /// narrowing is spelled.
    pub syntax: Vec<SyntaxProvider>,
    pub reachability: SubjectFactProvider,
    pub dependencies: SubjectFactsProvider<Launcher, Env>,
    pub lint: SubjectFactsProvider<Launcher, Env>,
    pub dependency_policy: WorkspaceFactProvider<Launcher, Env>,
    pub naming_policy: WorkspacePolicyProvider<Fs>,
    pub limits_policy: WorkspacePolicyProvider<Fs>,
    pub architecture: WorkspacePolicyProvider<Fs>,
    pub scripting_policy: WorkspacePolicyProvider<Fs>,
    pub goals_policy: WorkspacePolicyProvider<Fs>,
    pub words_policy: WorkspacePolicyProvider<Fs>,
    pub test_material_policy: WorkspacePolicyProvider<Fs>,
    pub requirement_trace: WorkspacePolicyProvider<Fs>,
}

/// Which composed `nomos.cap.syntax.items` provider `path` belongs to, if any does --
/// `OD-CAPABILITY-009`'s corrected fix, and the one function both halves of the pipeline
/// consult so they cannot independently drift on the answer.
///
/// `crate::run_context`'s own enrichment step calls this to populate
/// `nomos_rules::SourceFile::preferred_syntax_provider` before any rule sees a source, and
/// `crate::facts::Materialize_Syntax`'s write side calls it again over the identical path to
/// decide which provider's own materialization to run. Both call sites reach this one
/// function rather than each recomputing recognition for themselves, so read and write agree
/// on a subject's provider identity by construction.
///
/// It answers from the composed set rather than from two crate names, which is the whole of
/// what this item changed here: a third syntax provider becomes a row in
/// [`ComposedProviders::syntax`] and this function is untouched.
#[must_use]
pub fn Recognized_Syntax_Provider<'a>(providers: &'a [SyntaxProvider], path: &str) -> Option<&'a SyntaxProvider>
{
    for provider in providers
    {
        if (provider.recognizes)(path)
        {
            return Some(provider);
        }
    }

    return None;
}

/// Which language `path` is written in, or `None` if no composed provider recognizes it.
///
/// Sits beside [`Recognized_Syntax_Provider`] and asks the identical question of the
/// identical list, because a path's language and the provider that reads it are decided by
/// one recognition and must not be decided by two. `OD-RULES-014` moved this question out of
/// eleven private extension tests in `nomos-rules`; splitting it back across two lookups
/// here would restore the same defect one layer up.
///
/// It is deliberately not derived from [`Recognized_Syntax_Provider`]'s *identity*. That
/// mapping happens to be lossless today, but a provider identity names reading technology --
/// `nomos.lang.rust.syn` and `nomos.lang.rust.scan` are one language -- so deriving a
/// language from it would encode a many-to-one collapse a third Rust provider would silently
/// have to be added to. Each composed row carries its own language and this reads it.
#[must_use]
pub fn Recognized_Language(providers: &[SyntaxProvider], path: &str) -> Option<Language>
{
    return Recognized_Syntax_Provider(providers, path).map(|provider| return provider.language.clone());
}
