//! The one place in this crate that knows an analysis provider by name.
//!
//! [`Composed_Providers`] is a declared table: one row per capability this crate's
//! [`super::Registered`] offers a provider against, each row naming that provider's real
//! function and adapting it to the port its granularity requires. Everything else in the
//! crate reads the table. `OD-ROADMAP-005` decision item 1 authorizes that separation, and
//! `crate::composed_providers` carries the argument for why there is one port per granularity and not one port.
//!
//! # A declared row, never a condition
//!
//! `OD-ROADMAP-003`'s surviving constraint applies to this file directly: a provider
//! composition is a declared constant and never a condition consulting store state, cost or
//! prior materialization. Every row below is a function name and an adapter; none reads a
//! store, a clock, a count or another row. The sentence that would break it -- *offer this
//! provider when the other one would cost more* -- has no place to be written here, and if
//! one is ever wanted, `OD-RULES-009` is where it has to be argued.
//!
//! # The adaptation that happens here and nowhere else
//!
//! Each provider takes its own `FactContext`: four fields -- snapshot, variant,
//! configuration, generation -- copied straight off [`Context`]. `OD-CAPABILITY-008`
//! measured that this is one of the three parts of the provider convention that genuinely
//! agree across every provider in the workspace, and the crate used to prove it the hard
//! way, with twelve private `*_Production` functions in twelve materialization files, each
//! assigning the same four fields to a differently-named struct. Ten remain, one per
//! distinct `FactContext` type, and they all sit here.

use nomos_analysis::{Context, MaterializedFact};
use nomos_capability::{Registry, RegistryError};
use nomos_cap_syntax::Language;
use nomos_contracts::{ProviderId, SubjectId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use std::path::Path;

use crate::composed_providers::{WalkFacts, WalkReading, ComposedProviders, SitesProvider, SubjectFact, SyntaxProvider, Unanswered, WalkedSource};

/// Every analysis provider a run materializes through, composed.
///
/// One row per capability with a materialization step. `nomos.cap.review.finding` has none
/// and therefore no row: its provider answers about an already-identified external review
/// comment, and nothing in a walk over already-read source names one. `nomos.lang.rust.scan`
/// and `nomos.lang.go.modules` have none either -- both are real offers standing in the
/// registry against a capability another provider materializes for, which is what an offer
/// weaker than a caller's floor is for.
#[must_use]
pub(crate) fn Composed_Providers<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>() -> ComposedProviders<Launcher, Fs, Env>
{
    return ComposedProviders {
        syntax: Composed_Syntax_Providers(),
        sites: vec![
            SitesProvider { recognizes: Rust_Recognizes, materialize: Rust_Sites_Fact },
            SitesProvider { recognizes: Go_Recognizes, materialize: Go_Sites_Fact },
            SitesProvider { recognizes: Csharp_Recognizes, materialize: Csharp_Sites_Fact },
        ],
        reachability: Rust_Reachability_Fact,
        complexity: Rust_Complexity_Fact,
        dependencies: Cargo_Facts,
        lint: Clippy_Facts,
        walked_lint: vec![Go_Vet_Facts],
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
        csharp_conditional: Csharp_Conditional_Facts,
        go_discarded_values: Go_Types_Facts,
        offer: Offer_Composed_Providers,
    };
}

/// The registry offer of every provider the table above names, and of no other.
///
/// One line per offer, in the order the contracts are declared, so a provider added to the
/// table and forgotten here is a provider the registry refuses to resolve rather than one it
/// silently resolves to something else.
pub(crate) fn Offer_Composed_Providers(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Offer(nomos_lang_rust::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_scan::Provider_Offer())?;
    registry.Offer(nomos_lang_go::Provider_Offer())?;
    registry.Offer(nomos_lang_csharp::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_cargo::Provider_Offer())?;
    registry.Offer(nomos_lang_go_modules::Provider_Offer())?;
    registry.Offer(nomos_lang_rust::sites::Provider_Offer())?;
    registry.Offer(nomos_lang_go::sites::Provider_Offer())?;
    registry.Offer(nomos_lang_csharp::sites::Provider_Offer())?;
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
/// Declared as its own function rather than inline in [`Composed_Providers`]'s literal,
/// because the syntax half is the one part of the table that is free of the three port
/// types: a caller asking which provider reads a path -- `crate::run_context`'s own source
/// enrichment, and the write side of `crate::facts::Materialize_Syntax` -- needs no
/// launcher, no filesystem and no environment, and this crate's own proofs of the
/// recognition answer are written against it without naming three types they do not use.
///
/// `nomos_lang_rust_scan` is deliberately absent, exactly as it was before this table
/// existed: it is a real offer in the registry, weaker on every axis than the parser's, and
/// it has never been the provider a `.rs` file's fact is filed under. Adding it here would
/// change which identity that fact carries, which `OD-CAPABILITY-009` records the cost of.
#[must_use]
pub(crate) fn Composed_Syntax_Providers() -> Vec<SyntaxProvider>
{
    return vec![Rust_Syntax_Provider(), Go_Syntax_Provider(), Csharp_Syntax_Provider()];
}

/// `nomos_lang_rust`'s syntax offer, as the run holds it.
///
/// Its guarantee is read from the provider rather than restated: `crate::facts` rebuilds the
/// [`nomos_analysis::FactKey`] a materialization would produce, before parsing, to ask
/// whether the store already holds one -- and a guarantee stated twice would make that key
/// miss silently.
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

/// One source's `nomos.cap.syntax.sites` fact from `nomos_lang_rust`, or `None` if that provider
/// refused to parse it.
fn Rust_Sites_Fact(subject: SubjectId, source: &str, context: &Context) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_rust::Materialization::Materialized(fact) = nomos_lang_rust::sites::Materialize_Sites_Fact(subject, source, Rust_Production(context))
    else
    {
        return None;
    };

    return Some(fact);
}

/// One Go source's `nomos.cap.syntax.sites` fact from `nomos_lang_go`, or `None` if that provider
/// refused to parse it.
fn Go_Sites_Fact(subject: SubjectId, source: &str, context: &Context) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_go::Materialization::Materialized(fact) = nomos_lang_go::sites::Materialize_Sites_Fact(subject, source, Go_Production(context))
    else
    {
        return None;
    };

    return Some(fact);
}

/// One C# source's `nomos.cap.syntax.sites` fact from `nomos_lang_csharp`: its declines, which do not
/// wait on a parse.
fn Csharp_Sites_Fact(subject: SubjectId, source: &str, context: &Context) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_csharp::Materialization::Materialized(fact) = nomos_lang_csharp::sites::Materialize_Sites_Fact(subject, source, Csharp_Production(context))
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

/// Every walked C# file judged under every declared build that compiles it -- the copy of
/// `nomos-composer-providers`' own row, whose module doc argues each branch below.
fn Csharp_Conditional_Facts<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(reading: &WalkReading<'_, Launcher, Fs, Env>) -> WalkFacts
{
    use nomos_contracts::{Applicability, GateCategory};
    use nomos_repo_policy::csharp_builds::{CSHARP_BUILDS_JSON, Read_Declared_Builds};

    let files: Vec<nomos_lang_csharp_compiler::FileToJudge<'_>> =
        reading.sources.iter().filter(|source| return Csharp_Recognizes(source.path)).map(File_To_Judge).collect();
    if files.is_empty()
    {
        return WalkFacts::default();
    }
    let only = |subject_name: &str, applicability, gate, reason: String| {
        return WalkFacts { facts: Vec::new(), unanswered: vec![Unanswered { subject_name: subject_name.to_owned(), path: subject_name.to_owned(), applicability, gate, reason }] };
    };
    let builds = match Read_Declared_Builds(reading.root, reading.filesystem)
    {
        Ok(builds) if builds.is_empty() => return only(CSHARP_BUILDS_JSON, Applicability::NotApplicable, GateCategory::Advisory, format!("this repository has {} C# source(s) and declares no build in {CSHARP_BUILDS_JSON}, so none of their conditional branches was judged: which branch compiles turns on a build's symbols, and no build is guessed", files.len())),
        Ok(builds) => builds,
        Err(error) => return only(CSHARP_BUILDS_JSON, Applicability::Unparseable, GateCategory::Blocking, format!("{error}; no C# conditional branch is judged until it is corrected")),
    };

    let requests: Vec<nomos_lang_csharp_compiler::EvaluationRequest> = builds
        .into_iter()
        .map(|build| return nomos_lang_csharp_compiler::EvaluationRequest { root: reading.root.to_path_buf(), project: build.project, configuration: build.configuration, target_framework: build.target_framework })
        .collect();
    let ports = nomos_lang_csharp_compiler::BuildPorts { launcher: reading.launcher, environment: reading.environment };
    let answer = nomos_lang_csharp_compiler::Materialize_Build_Set(&requests, &files, Conditional_Production(reading.context), &ports);

    return WalkFacts {
        facts: answer.facts.into_iter().map(|judged| return SubjectFact { path: judged.path, subject: judged.fact.Key().subject, fact: judged.fact }).collect(),
        unanswered: answer.unjudged.into_iter().map(Csharp_Unanswered).collect(),
    };
}

fn File_To_Judge<'text>(source: &WalkedSource<'text>) -> nomos_lang_csharp_compiler::FileToJudge<'text>
{
    return nomos_lang_csharp_compiler::FileToJudge { path: source.path, subject: source.subject, text: source.text };
}

fn Csharp_Unanswered(unjudged: nomos_lang_csharp_compiler::Unjudged) -> Unanswered
{
    use nomos_contracts::{Applicability, GateCategory};

    return match unjudged
    {
        nomos_lang_csharp_compiler::Unjudged::Build { request, error } =>
        {
            let framework = request.target_framework.unwrap_or_else(|| return "its one framework".to_owned());
            Unanswered {
                applicability: error.failure.Applicability(),
                gate: GateCategory::Advisory,
                reason: format!(
                    "the declared build {} {} {framework} could not be evaluated, so no C# conditional branch was judged against any declared build --                      a branch only this build compiles would otherwise read as compiled by nothing: {error}",
                    request.project, request.configuration
                ),
                path: request.project.clone(),
                subject_name: request.project,
            }
        }
        nomos_lang_csharp_compiler::Unjudged::File { path, failure } => Unanswered {
            subject_name: path.clone(),
            path,
            applicability: Applicability::Unparseable,
            gate: GateCategory::Advisory,
            reason: format!("this file's preprocessor directives could not be read, so its conditional branches were not judged against any declared build: {failure}"),
        },
    };
}

/// Every walked Go source linted, module by module, by `go vet` -- the copy of
/// `nomos-composer-providers`' own row, whose module doc argues each branch below.
fn Go_Vet_Facts<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(reading: &WalkReading<'_, Launcher, Fs, Env>) -> WalkFacts
{
    use nomos_contracts::{Applicability, GateCategory};
    use nomos_lang_go_lint::Unlinted;

    let files: Vec<&str> = reading.sources.iter().filter(|source| return Go_Recognizes(source.path)).map(|source| return source.path).collect();
    let ports = nomos_lang_go_lint::LintPorts { launcher: reading.launcher, filesystem: reading.filesystem, environment: reading.environment };
    let context = nomos_lang_go_lint::FactContext { snapshot: reading.context.snapshot, variant: reading.context.variant, configuration: reading.context.configuration, generation: reading.context.generation };
    let answer = nomos_lang_go_lint::Materialize_Modules(reading.root, &files, context, &ports);

    let unanswered = answer.unlinted.into_iter().map(|unlinted| {
        return match unlinted
        {
            Unlinted::Module { path, failure, reason } =>
            {
                let manifest = if path.is_empty() { "go.mod".to_owned() } else { format!("{path}/go.mod") };
                Unanswered {
                    reason: format!("go vet gave no answer for the Go module at {manifest}, so its lint diagnostics were not judged: {reason}"),
                    subject_name: manifest,
                    path,
                    applicability: failure.Applicability(),
                    gate: GateCategory::Advisory,
                }
            }
            Unlinted::Outside { files } =>
            {
                let first = files.first().cloned().unwrap_or_default();
                Unanswered {
                    reason: format!("{} Go source(s) sit under no go.mod, so no module holds them and go vet has no package to load them in", files.len()),
                    subject_name: first.clone(),
                    path: first,
                    applicability: Applicability::NotApplicable,
                    gate: GateCategory::Advisory,
                }
            }
        };
    });

    return WalkFacts {
        facts: answer.facts.into_iter().map(|module| return SubjectFact { path: module.path, subject: module.subject, fact: module.fact }).collect(),
        unanswered: unanswered.collect(),
    };
}

/// Every walked Go source typed, module by module, by the Go types helper -- the copy of
/// `nomos-composer-providers`' own row, whose module doc argues each branch below.
fn Go_Types_Facts<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(reading: &WalkReading<'_, Launcher, Fs, Env>) -> WalkFacts
{
    use nomos_contracts::{Applicability, GateCategory};
    use nomos_lang_go_types::Untyped;

    let files: Vec<&str> = reading.sources.iter().filter(|source| return Go_Recognizes(source.path)).map(|source| return source.path).collect();
    let ports = nomos_lang_go_types::TypesPorts { launcher: reading.launcher, filesystem: reading.filesystem, environment: reading.environment };
    let context = nomos_lang_go_types::FactContext { snapshot: reading.context.snapshot, variant: reading.context.variant, configuration: reading.context.configuration, generation: reading.context.generation };
    let answer = nomos_lang_go_types::Materialize_Files(reading.root, &files, context, &ports);

    let unanswered = answer.untyped.into_iter().map(|untyped| {
        let (path, applicability, reason) = match untyped
        {
            Untyped::Module { path, failure, reason } => (format!("{path}/go.mod").trim_start_matches('/').to_owned(), failure.Applicability(), format!("the Go type checker gave no answer for this module: {reason}")),
            Untyped::Package { package, files, reason } => (files.first().cloned().unwrap_or_default(), Applicability::AnalysisFailed, format!("package {package} does not type-check: {reason}")),
            Untyped::Unchecked { files, .. } => (files.first().cloned().unwrap_or_default(), Applicability::NotApplicable, format!("{} Go source(s) are compiled by no package on this host", files.len())),
            Untyped::Outside { files } => (files.first().cloned().unwrap_or_default(), Applicability::NotApplicable, format!("{} Go source(s) sit under no go.mod", files.len())),
        };
        return Unanswered { subject_name: path.clone(), path, applicability, gate: GateCategory::Advisory, reason };
    });

    return WalkFacts {
        facts: answer.facts.into_iter().map(|file| return SubjectFact { path: file.path, subject: file.subject, fact: file.fact }).collect(),
        unanswered: unanswered.collect(),
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
    use super::*;
    use crate::composed_providers::{Recognized_Language, Recognized_Syntax_Provider};

    /// [`Recognized_Syntax_Provider`]'s whole contract over the composed table: a `.rs` path
    /// resolves to `nomos_lang_rust`'s identity, a `.go` path to `nomos_lang_go`'s, and a
    /// path neither recognizes resolves to neither.
    ///
    /// It sits here rather than beside the lookup it exercises because this is the module
    /// that knows which crate each identity belongs to; the lookup itself names none.
    #[test]
    fn Test_Recognized_Syntax_Provider_Should_Resolve_By_Extension()
    {
        let composed = Composed_Syntax_Providers();

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

    /// [`Recognized_Language`]'s whole contract, and deliberately a separate assertion from
    /// the provider one above: the two answers come from the same recognition but are not
    /// the same fact.
    #[test]
    fn Test_Recognized_Language_Should_Resolve_By_Extension()
    {
        let composed = Composed_Syntax_Providers();

        assert_eq!(Recognized_Language(&composed, "a.rs"), Some(Language::New(nomos_lang_rust::LANGUAGE)));
        assert_eq!(Recognized_Language(&composed, "main.go"), Some(Language::New(nomos_lang_go::LANGUAGE)));
        assert_eq!(Recognized_Language(&composed, "Program.cs"), Some(Language::New(nomos_lang_csharp::LANGUAGE)));
        assert_eq!(Recognized_Language(&composed, "readme.md"), None);
    }
}
