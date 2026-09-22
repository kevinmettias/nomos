//! The one place in this crate that knows an analysis provider by name.
//!
//! [`Composed_Providers`] is a declared table: one row per capability this crate's
//! [`super::Registered`] offers a provider against, each row naming that provider's real
//! function and adapting it to the port its granularity requires. Everything else in the
//! crate reads the table. `OD-ROADMAP-005` decision item 1 authorizes that separation, and
//! `crate::composed_providers` carries the argument for why the ports are five and not one.
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
//! assigning the same four fields to a differently-named struct. Seven remain, one per
//! distinct `FactContext` type, and they all sit here.

use nomos_analysis::{Context, MaterializedFact};
use nomos_cap_syntax::Language;
use nomos_contracts::{ProviderId, SubjectId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use std::path::Path;

use crate::composed_providers::{ComposedProviders, SubjectFact, SyntaxProvider};

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
        reachability: Rust_Reachability_Fact,
        dependencies: Cargo_Facts,
        lint: Clippy_Facts,
        dependency_policy: Deny_Fact,
        naming_policy: Naming_Policy_Fact,
        limits_policy: Limits_Policy_Fact,
        architecture: Architecture_Fact,
        scripting_policy: Scripting_Policy_Fact,
        goals_policy: Goals_Policy_Fact,
        words_policy: Words_Policy_Fact,
        test_material_policy: Test_Material_Policy_Fact,
        requirement_trace: Requirement_Trace_Fact,
    };
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
    return vec![Rust_Syntax_Provider(), Go_Syntax_Provider()];
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
        assert_eq!(Recognized_Language(&composed, "readme.md"), None);
    }

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
