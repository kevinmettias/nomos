//! Zone: Composer -- the standard provider set, selected once.
//!
//! [`Standard_Providers`] builds the [`ComposedProviders`] value `nomos-check-orchestration`
//! runs through: one row per capability, each naming a provider's real function and adapting it
//! to the port its granularity requires, plus the registry offers of exactly those providers.
//! `OD-HOST-020` decided that this selection is made here, once, and nowhere else.
//!
//! # What this crate is, and what it is not
//!
//! It **selects**, in the sense `nomos-composer-std` gives that word one layer down: that crate
//! is where "this workspace runs on the ordinary operating system" is written once, and this
//! one is where "this workspace analyses with its standard providers" is. It **orchestrates
//! nothing** -- no verb, no sequence, and no type that exists to be rendered. It is a library,
//! not a composition root; roots name it.
//!
//! The service does not. `nomos-check-orchestration` receives the set through `RunContext` and
//! names no language, repository-policy or connector provider in its library dependency graph.
//! `OD-ROADMAP-005` decision item 1 authorized that, and `OD-HOST-001` had already settled that
//! naming a concrete provider in a composition root was never the defect.
//!
//! # Who may name this crate
//!
//! The four hosts do: `nomos-cli`, `nomos-api`, `nomos-lsp` and `nomos-daemon`. The
//! Application Service crates that run the check on a host's behalf -- gate, correction and
//! workflow orchestration -- do **not**; they receive the set through the context structs they
//! already carry and pass it on. A service that called this crate would be choosing the
//! standard set on its caller's behalf, which is the composition root disguised as an
//! application service that the move exists to remove, one crate over. `nomos-architecture.json`
//! enforces it: `Host` may reach `Composer` and `Application Service` may not.
//!
//! # The table and the offers are one list
//!
//! Before this crate the registry offers lived in the service's `Registered` and the rows in its
//! provider table, one choice made twice with nothing keeping the two in agreement. Here they are
//! built side by side, and the value carries both, so a provider added to the table and not
//! offered is one the registry refuses to resolve rather than one it resolves to something else.
//!
//! # Why this crate depends on the service
//!
//! `ComposedProviders` and its port types stay in `nomos-check-orchestration`. A crate holding
//! them below both would need `nomos-cap-syntax` for `SyntaxProvider`'s language, and a
//! Capability Contract crate may not reach another. `OD-HOST-020` section 5 records the cost:
//! the service's own unit tests keep one local table, because a dev-dependency on this crate,
//! which depends back on them, would hand them a `ComposedProviders` of a different type.
//!
//! # A declared row, never a condition
//!
//! `OD-ROADMAP-003`'s surviving constraint applies to this crate directly: a provider
//! composition is a declared constant and never a condition consulting store state, cost or
//! prior materialization. Every row below is a function name and an adapter; none reads a
//! store, a clock, a count or another row. The sentence that would break it -- *offer this
//! provider when the other one would cost more* -- has no place to be written here, and if one
//! is ever wanted, `OD-RULES-009` is where it has to be argued.
//!
//! # The adaptation that happens here and nowhere else
//!
//! Each provider takes its own `FactContext`: four fields -- snapshot, variant, configuration,
//! generation -- copied straight off [`Context`]. `OD-CAPABILITY-008` measured that this is one
//! of the three parts of the provider convention that genuinely agree across every provider in
//! the workspace. There is one `*_Production` function per distinct `FactContext` type, and they
//! all sit here.

#![forbid(unsafe_code)]

mod csharp_conditional;
mod go_lint;
mod go_types;

use nomos_analysis::{Context, MaterializedFact};
use nomos_capability::{Registry, RegistryError};
use nomos_cap_syntax::Language;
use nomos_contracts::{ProviderId, SubjectId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use std::path::Path;

use nomos_check_orchestration::{ComposedProviders, SubjectFact, SyntaxProvider};

/// Every analysis provider a run materializes through, composed.
///
/// One row per capability with a materialization step. `nomos.cap.review.finding` has none
/// and therefore no row: its provider answers about an already-identified external review
/// comment, and nothing in a walk over already-read source names one. `nomos.lang.rust.scan`
/// and `nomos.lang.go.modules` have none either -- both are real offers standing in the
/// registry against a capability another provider materializes for, which is what an offer
/// weaker than a caller's floor is for.
#[must_use]
pub fn Standard_Providers<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>() -> ComposedProviders<Launcher, Fs, Env>
{
    return ComposedProviders {
        syntax: Composed_Syntax_Providers(),
        reachability: Rust_Reachability_Fact,
        complexity: Rust_Complexity_Fact,
        dependencies: Cargo_Facts,
        lint: Clippy_Facts,
        walked_lint: vec![go_lint::Go_Vet_Facts],
        dependency_policy: Deny_Fact,
        naming_policy: Naming_Policy_Fact,
        limits_policy: Limits_Policy_Fact,
        architecture: Architecture_Fact,
        scripting_policy: Scripting_Policy_Fact,
        goals_policy: Goals_Policy_Fact,
        words_policy: Words_Policy_Fact,
        test_material_policy: Test_Material_Policy_Fact,
        requirement_trace: Requirement_Trace_Fact,
        standards_corpus_policy: Standards_Corpus_Policy_Fact,
        copy_clones: Copy_Clones_Fact,
        nested_locks: Nested_Locks_Fact,
        csharp_conditional: csharp_conditional::Csharp_Conditional_Facts,
        go_discarded_values: go_types::Go_Types_Facts,
        offer: Standard_Offers,
    };
}

/// The registry offer of every provider [`Standard_Providers`]'s table names, and of no other.
///
/// Public on its own for a caller that needs the registry without a run -- `nomos-cli`'s
/// `profile` reads which capability each provider offers against, and builds no table. It is
/// the same function the table carries in its `offer` field, so the two cannot disagree.
///
/// # Errors
///
/// Whatever [`Registry::Offer`] returns for the first offer it refuses -- a contract not yet
/// declared, a provider offered twice, or a guarantee above the contract's ceiling. The service
/// declares every contract before calling this, so on this workspace's own set it does not err;
/// `nomos-check-orchestration` maps any refusal to `CheckOutcome::Contradictory`.
///
/// One line per offer, in the order the contracts are declared, so a provider added to the
/// table and forgotten here is a provider the registry refuses to resolve rather than one it
/// silently resolves to something else.
pub fn Standard_Offers(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Offer(nomos_lang_rust::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_scan::Provider_Offer())?;
    registry.Offer(nomos_lang_go::Provider_Offer())?;
    registry.Offer(nomos_lang_csharp::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_cargo::Provider_Offer())?;
    registry.Offer(nomos_lang_go_modules::Provider_Offer())?;
    registry.Offer(nomos_lang_rust::reachability::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_complexity::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_clippy::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_deny::Provider_Offer())?;
    registry.Offer(nomos_repo_policy::naming::Provider_Offer())?;
    registry.Offer(nomos_repo_policy::limits::Provider_Offer())?;
    registry.Offer(nomos_repo_policy::architecture::Provider_Offer())?;
    registry.Offer(nomos_repo_policy::scripting::Provider_Offer())?;
    registry.Offer(nomos_repo_policy::goals::Provider_Offer())?;
    registry.Offer(nomos_repo_policy::words::Provider_Offer())?;
    registry.Offer(nomos_repo_policy::test_material::Provider_Offer())?;
    registry.Offer(nomos_connector_coderabbit::Provider_Offer())?;
    registry.Offer(nomos_cap_requirement_trace::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_compiler::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_compiler::Nested_Locks_Provider_Offer())?;
    registry.Offer(nomos_repo_policy::standards_corpus::Provider_Offer())?;
    registry.Offer(nomos_lang_csharp_compiler::Provider_Offer())?;
    registry.Offer(nomos_lang_go_lint::Provider_Offer())?;
    registry.Offer(nomos_lang_go_types::Provider_Offer())?;

    return Ok(());
}

/// Every composed `nomos.cap.syntax.items` offer, in the order a recognition question
/// consults them.
///
/// Declared as its own function rather than inline in [`Standard_Providers`]'s literal,
/// because the syntax half is the one part of the table that is free of the three port
/// types: a caller asking which provider reads a path -- `nomos-check-orchestration`'s own
/// source enrichment, and the write side of its `Materialize_Syntax` -- needs no
/// launcher, no filesystem and no environment, and this crate's own proof that the list
/// resolves each language is written against it without naming three types it does not use.
///
/// `nomos_lang_rust_scan` is deliberately absent, exactly as it was before this table
/// existed: it is a real offer in the registry, weaker on every axis than the parser's, and
/// it has never been the provider a `.rs` file's fact is filed under. Adding it here would
/// change which identity that fact carries, which `OD-CAPABILITY-009` records the cost of.
#[must_use]
fn Composed_Syntax_Providers() -> Vec<SyntaxProvider>
{
    return vec![Rust_Syntax_Provider(), Go_Syntax_Provider(), Csharp_Syntax_Provider()];
}

/// `nomos_lang_rust`'s syntax offer, as the run holds it.
///
/// Its guarantee is read from the provider rather than restated: `nomos-check-orchestration`
/// rebuilds the [`nomos_analysis::FactKey`] a materialization would produce, before parsing, to ask
/// whether the store already holds one -- and a guarantee stated twice would make that key miss
/// silently.
fn Rust_Syntax_Provider() -> SyntaxProvider
{
    return SyntaxProvider {
        provider: ProviderId::New(nomos_lang_rust::PROVIDER),
        language: Language::New(nomos_lang_rust::LANGUAGE),
        guarantee: nomos_lang_rust::Declared_Guarantee(),
        recognizes: Rust_Recognizes,
        materialize: Rust_Syntax_Fact,
    };
}

/// `nomos_lang_go`'s syntax offer, as the run holds it -- the identical five values
/// [`Rust_Syntax_Provider`] states, read from the other provider.
fn Go_Syntax_Provider() -> SyntaxProvider
{
    return SyntaxProvider {
        provider: ProviderId::New(nomos_lang_go::PROVIDER),
        language: Language::New(nomos_lang_go::LANGUAGE),
        guarantee: nomos_lang_go::Declared_Guarantee(),
        recognizes: Go_Recognizes,
        materialize: Go_Syntax_Fact,
    };
}

/// `nomos_lang_csharp`'s syntax offer, as the run holds it -- the same five values again,
/// read from the third provider.
fn Csharp_Syntax_Provider() -> SyntaxProvider
{
    return SyntaxProvider {
        provider: ProviderId::New(nomos_lang_csharp::PROVIDER),
        language: Language::New(nomos_lang_csharp::LANGUAGE),
        guarantee: nomos_lang_csharp::Declared_Guarantee(),
        recognizes: Csharp_Recognizes,
        materialize: Csharp_Syntax_Fact,
    };
}

/// Whether `nomos_lang_rust` recognizes `path`.
fn Rust_Recognizes(path: &str) -> bool
{
    return nomos_lang_rust::Recognition::Of_Path(path) == nomos_lang_rust::Recognition::Recognized;
}

/// Whether `nomos_lang_go` recognizes `path`.
fn Go_Recognizes(path: &str) -> bool
{
    return nomos_lang_go::Recognition::Of_Path(path) == nomos_lang_go::Recognition::Recognized;
}

/// Whether `nomos_lang_csharp` recognizes `path`.
fn Csharp_Recognizes(path: &str) -> bool
{
    return nomos_lang_csharp::Recognition::Of_Path(path) == nomos_lang_csharp::Recognition::Recognized;
}

/// One source's `nomos.cap.syntax.items` fact from `nomos_lang_rust`, or `None` if that
/// provider refused to parse it.
fn Rust_Syntax_Fact(subject: SubjectId, source: &str, context: &Context) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_rust::Materialization::Materialized(fact) = nomos_lang_rust::Materialize_Syntax_Fact(subject, source, Rust_Production(context))
    else
    {
        return None;
    };

    return Some(fact);
}

/// One source's `nomos.cap.syntax.items` fact from `nomos_lang_go`, or `None` if that
/// provider refused to parse it.
fn Go_Syntax_Fact(subject: SubjectId, source: &str, context: &Context) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_go::Materialization::Materialized(fact) = nomos_lang_go::Materialize_Syntax_Fact(subject, source, Go_Production(context))
    else
    {
        return None;
    };

    return Some(fact);
}

/// One source's `nomos.cap.controlflow.reachability` fact from `nomos_lang_rust`'s tier-1
/// heuristic provider, or `None` if it refused.
fn Rust_Reachability_Fact(subject: SubjectId, source: &str, context: &Context) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_rust::Materialization::Materialized(fact) =
        nomos_lang_rust::reachability::Materialize_Reachability_Fact(subject, source, Rust_Production(context))
    else
    {
        return None;
    };

    return Some(fact);
}

/// Every workspace member's `nomos.cap.dependency.edges` fact, from `cargo metadata`.
///
/// The error is rendered here because rendering is the only thing this crate has ever done
/// with it: the caller turns the text into one `ProviderUnavailable` finding's summary.
fn Cargo_Facts<Launcher: ProgramLauncher, Env: Environment>(
    root: &Path, context: &Context, launcher: &Launcher, environment: &Env,
) -> Result<Vec<SubjectFact>, String>
{
    let facts = nomos_lang_rust_cargo::Materialize_Workspace(root, Cargo_Production(context), launcher, environment)
        .map_err(|error| return error.to_string())?;

    return Ok(facts.into_iter().map(|package| return SubjectFact { path: package.path, subject: package.subject, fact: package.fact }).collect());
}

/// Every workspace member's `nomos.cap.lint.diagnostics` fact, from `cargo clippy`.
fn Clippy_Facts<Launcher: ProgramLauncher, Env: Environment>(
    root: &Path, context: &Context, launcher: &Launcher, environment: &Env,
) -> Result<Vec<SubjectFact>, String>
{
    let facts = nomos_lang_rust_clippy::Materialize_Workspace(root, Clippy_Production(context), launcher, environment)
        .map_err(|error| return error.to_string())?;

    return Ok(facts.into_iter().map(|member| return SubjectFact { path: member.path, subject: member.subject, fact: member.fact }).collect());
}

/// The workspace's one `nomos.cap.dependency.policy` fact, from `cargo deny`.
///
/// The path is the literal this capability has always filed under: its declared ceiling is
/// `IncrementalGranularity::WholeWorkspace`, so a bans/licenses/sources verdict is not
/// attributable to any one member and the provider states no path of its own.
fn Deny_Fact<Launcher: ProgramLauncher, Env: Environment>(
    root: &Path, context: &Context, launcher: &Launcher, environment: &Env,
) -> Result<SubjectFact, String>
{
    let policy = nomos_lang_rust_deny::Materialize_Workspace(root, Deny_Production(context), launcher, environment)
        .map_err(|error| return error.to_string())?;

    return Ok(SubjectFact { path: "workspace".to_owned(), subject: policy.subject, fact: policy.fact });
}

/// The workspace's one `nomos.cap.naming.policy` fact, read from `standards.json`.
fn Naming_Policy_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return nomos_repo_policy::naming::Materialize_Workspace(root, Repo_Policy_Production(context), filesystem).ok().map(|policy| return policy.fact);
}

/// The workspace's one `nomos.cap.limits.policy` fact, read from `standards.json`.
fn Limits_Policy_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return nomos_repo_policy::limits::Materialize_Workspace(root, Repo_Policy_Production(context), filesystem).ok().map(|policy| return policy.fact);
}

/// The workspace's one `nomos.cap.architecture.declaration` fact, read from
/// `nomos-architecture.json` rather than from `standards.json` -- that file is shared with a
/// tool decoding it with unknown fields disallowed, so the one key this repository would own
/// outright is not available in it.
fn Architecture_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return nomos_repo_policy::architecture::Materialize_Workspace(root, Repo_Policy_Production(context), filesystem).ok().map(|policy| return policy.fact);
}

/// The workspace's one `nomos.cap.scripting.policy` fact, read from `standards.json`.
fn Scripting_Policy_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return nomos_repo_policy::scripting::Materialize_Workspace(root, Repo_Policy_Production(context), filesystem).ok().map(|policy| return policy.fact);
}

/// The workspace's one `nomos.cap.goals.policy` fact, read from `standards.json`.
fn Goals_Policy_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return nomos_repo_policy::goals::Materialize_Workspace(root, Repo_Policy_Production(context), filesystem).ok().map(|policy| return policy.fact);
}

/// The workspace's one `nomos.cap.words.policy` fact, read from `standards.json`.
fn Words_Policy_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return nomos_repo_policy::words::Materialize_Workspace(root, Repo_Policy_Production(context), filesystem).ok().map(|policy| return policy.fact);
}

/// The workspace's one `nomos.cap.test.material.policy` fact, read from
/// `nomos-test-material.json`.
fn Test_Material_Policy_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return nomos_repo_policy::test_material::Materialize_Workspace(root, Repo_Policy_Production(context), filesystem)
        .ok()
        .map(|policy| return policy.fact);
}

/// The workspace's one `nomos.cap.requirement.trace` fact, read from
/// `tests/contract/requirements/`.
///
/// The one row here that cannot fail. A missing requirements directory is this capability's
/// ordinary case -- every repository this rule judges except this one -- rather than a read
/// failure, so its provider is infallible by its own design and the `Option` this port
/// returns is always `Some`.
fn Requirement_Trace_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return Some(nomos_cap_requirement_trace::Materialize_Workspace(root, Requirement_Trace_Production(context), filesystem).fact);
}

/// The workspace's one `nomos.cap.standards.corpus` fact, read from
/// `nomos-standards-corpus.json` and every markdown document under the roots it declares.
///
/// The only row whose population is not the workspace being checked: the reader walks a
/// corpus the repository points at -- today the sibling `code-standards` repository's, which
/// is where those 1,133 documents are declared and where `ARC-CONFORMANCE-003` decided they
/// stay. A repository declaring no corpus is not a read failure: its declaration file is
/// absent, the reader answers an empty population, and this row produces a fact like any
/// other, which is what keeps a rule's verdict identical to what it was before this
/// capability existed.
fn Standards_Corpus_Policy_Fact<Fs: FileSystem>(root: &Path, context: &Context, filesystem: &Fs) -> Option<MaterializedFact>
{
    return nomos_repo_policy::standards_corpus::Materialize_Workspace(root, Repo_Policy_Production(context), filesystem)
        .ok()
        .map(|policy| return policy.fact);
}

/// The analyzed project's one `nomos.cap.rust.copy_clones` fact, from `ra_ap_hir`.
///
/// The path is the literal this capability files under for the same reason
/// [`Deny_Fact`]'s is: the provider analyzes the project rooted at the path a check was asked
/// about and states no path of its own, so the caller supplies the one name that is true of
/// whatever it was handed. The subject is the provider's, not this literal's -- a rule reads
/// the fact under the identity the provider filed it with.
fn Copy_Clones_Fact<Env: Environment>(root: &Path, context: &Context, environment: &Env) -> Result<SubjectFact, String>
{
    let analyzed = nomos_lang_rust_compiler::Materialize_Crate(root, Compiler_Production(context), environment)
        .map_err(|error| return error.to_string())?;

    return Ok(SubjectFact { path: "workspace".to_owned(), subject: analyzed.subject, fact: analyzed.fact });
}

/// The analyzed project's one `nomos.cap.rust.nested_locks` fact, from the same `ra_ap_hir`
/// engine [`Copy_Clones_Fact`] one row above runs, asked a different question.
fn Nested_Locks_Fact<Env: Environment>(root: &Path, context: &Context, environment: &Env) -> Result<SubjectFact, String>
{
    let analyzed = nomos_lang_rust_compiler::Materialize_Nested_Locks(root, Compiler_Production(context), environment)
        .map_err(|error| return error.to_string())?;

    return Ok(SubjectFact { path: "workspace".to_owned(), subject: analyzed.subject, fact: analyzed.fact });
}

/// The reading context both of `nomos_lang_rust_compiler`'s providers take.
///
/// One function for two capabilities, not two, for the same reason
/// [`Repo_Policy_Production`] is one function for seven: the two providers are one crate and
/// share a single `FactContext` type.
fn Compiler_Production(context: &Context) -> nomos_lang_rust_compiler::FactContext
{
    return nomos_lang_rust_compiler::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_lang_rust`'s own providers take it -- both the syntax one
/// and the reachability one, which share a `FactContext` because they are one crate.
fn Rust_Production(context: &Context) -> nomos_lang_rust::FactContext
{
    return nomos_lang_rust::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// One source's `nomos.cap.metric.complexity` fact from `nomos_lang_rust_complexity`, or `None`
/// if it refused to parse the source.
fn Rust_Complexity_Fact(subject: SubjectId, source: &str, context: &Context) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_rust_complexity::Materialization::Materialized(fact) =
        nomos_lang_rust_complexity::Materialize_Complexity_Fact(subject, source, Complexity_Production(context))
    else
    {
        return None;
    };

    return Some(fact);
}

/// The reading context as `nomos_lang_rust_complexity`'s own provider takes it.
fn Complexity_Production(context: &Context) -> nomos_lang_rust_complexity::FactContext
{
    return nomos_lang_rust_complexity::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_lang_go`'s own provider takes it.
fn Go_Production(context: &Context) -> nomos_lang_go::FactContext
{
    return nomos_lang_go::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// One source's `nomos.cap.syntax.items` fact from `nomos_lang_csharp`, or `None` if that
/// provider refused to parse it.
fn Csharp_Syntax_Fact(subject: SubjectId, source: &str, context: &Context) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_csharp::Materialization::Materialized(fact) =
        nomos_lang_csharp::Materialize_Syntax_Fact(subject, source, Csharp_Production(context))
    else
    {
        return None;
    };

    return Some(fact);
}

/// The reading context as `nomos_lang_csharp`'s own provider takes it.
fn Csharp_Production(context: &Context) -> nomos_lang_csharp::FactContext
{
    return nomos_lang_csharp::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_lang_go_lint`'s own provider takes it.
fn Go_Lint_Production(context: &Context) -> nomos_lang_go_lint::FactContext
{
    return nomos_lang_go_lint::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_lang_go_types`' own provider takes it.
fn Go_Types_Production(context: &Context) -> nomos_lang_go_types::FactContext
{
    return nomos_lang_go_types::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_lang_csharp_compiler`'s own provider takes it.
fn Conditional_Production(context: &Context) -> nomos_lang_csharp_compiler::FactContext
{
    return nomos_lang_csharp_compiler::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_lang_rust_cargo`'s own provider takes it.
fn Cargo_Production(context: &Context) -> nomos_lang_rust_cargo::FactContext
{
    return nomos_lang_rust_cargo::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_lang_rust_clippy`'s own provider takes it.
fn Clippy_Production(context: &Context) -> nomos_lang_rust_clippy::FactContext
{
    return nomos_lang_rust_clippy::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_lang_rust_deny`'s own provider takes it.
fn Deny_Production(context: &Context) -> nomos_lang_rust_deny::FactContext
{
    return nomos_lang_rust_deny::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context every `nomos_repo_policy` family takes.
///
/// One function for seven families, not seven, because that crate's own `scaffolding` module
/// declares the type once and each family re-exports it: `nomos_repo_policy::naming::
/// FactContext` and `nomos_repo_policy::architecture::FactContext` are two names for one
/// struct. Naming it through `naming` is arbitrary and any family's re-export would compile;
/// what is not arbitrary is that seven copies of this function would be seven copies of one
/// type's construction.
fn Repo_Policy_Production(context: &Context) -> nomos_repo_policy::naming::FactContext
{
    return nomos_repo_policy::naming::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The reading context as `nomos_cap_requirement_trace`'s own provider takes it -- its own
/// type, because that capability bundles its contract with its one provider in a crate of
/// its own rather than in `nomos-repo-policy`.
fn Requirement_Trace_Production(context: &Context) -> nomos_cap_requirement_trace::FactContext
{
    return nomos_cap_requirement_trace::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

#[cfg(test)]
mod tests
{
    use nomos_check_orchestration::Recognized_Syntax_Provider;
    use nomos_contracts::ProviderId;

    /// The guard `OD-RULES-014` requires, and the one place in this workspace that can hold
    /// it. A language-restricted rule states its own literal because `nomos-rules` may not
    /// depend on a language crate, so nothing in that crate can check the literal against
    /// what the provider actually declares. If the two ever drift the rule simply stops
    /// firing -- no findings, no reported absence -- which is exactly the silent failure the
    /// record set out to remove. This module depends on both sides and fails loudly instead.
    #[test]
    fn Test_The_Language_Names_Rules_State_Should_Agree_With_What_The_Language_Crates_Declare()
    {
        assert_eq!(nomos_rules::RUST_LANGUAGE, nomos_lang_rust::LANGUAGE);
        assert_eq!(nomos_rules::RUST_LANGUAGE, nomos_lang_rust_scan::LANGUAGE);
        assert_eq!(nomos_rules::GO_LANGUAGE, nomos_lang_go::LANGUAGE);
        assert_eq!(nomos_rules::CSHARP_LANGUAGE, nomos_lang_csharp::LANGUAGE);
    }

    /// The standard syntax list resolves each language it composes to that language's own
    /// provider, and a path none of them recognizes to none.
    ///
    /// `nomos-check-orchestration`'s unit tests prove the same lookup against their own local
    /// table, which `OD-HOST-020` section 5 keeps and which is a copy rather than this list.
    /// This is the proof over the list every real run holds. It is needed because the syntax
    /// half is a list and not a field per provider: a row dropped from it still compiles, and
    /// every file of that language would then go unparsed rather than fail to build.
    #[test]
    fn Test_The_Standard_Syntax_List_Should_Resolve_Each_Composed_Language_To_Its_Own_Provider()
    {
        let composed = super::Composed_Syntax_Providers();

        assert_eq!(
            Recognized_Syntax_Provider(&composed, "a.rs").map(|provider| return provider.provider.clone()),
            Some(ProviderId::New(nomos_lang_rust::PROVIDER))
        );
        assert_eq!(
            Recognized_Syntax_Provider(&composed, "main.go").map(|provider| return provider.provider.clone()),
            Some(ProviderId::New(nomos_lang_go::PROVIDER))
        );
        assert_eq!(
            Recognized_Syntax_Provider(&composed, "Program.cs").map(|provider| return provider.provider.clone()),
            Some(ProviderId::New(nomos_lang_csharp::PROVIDER))
        );
        assert!(Recognized_Syntax_Provider(&composed, "readme.md").is_none());
    }

    /// Both Rust providers are one language, which is the measurement that decided
    /// `OD-RULES-014`: a provider identity cannot stand in for a language because this
    /// equality holds while the identities differ.
    #[test]
    fn Test_The_Two_Rust_Providers_Should_Declare_One_Language_Under_Two_Identities()
    {
        assert_eq!(nomos_lang_rust::LANGUAGE, nomos_lang_rust_scan::LANGUAGE);
        assert_ne!(nomos_lang_rust::PROVIDER, nomos_lang_rust_scan::PROVIDER);
    }
}
