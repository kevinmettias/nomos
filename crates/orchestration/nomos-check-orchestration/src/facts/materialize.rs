//! Materializing facts from already-walked source: syntax, reachability and dependency
//! edges.
//!
//! Grouped by what they share -- pure, per-source materialization over a [`Context`] this
//! module does not build -- not by the capability each one answers; see each function's own
//! doc for why the three stayed independent steps rather than one generalization.

mod lint_materialization;
mod policy_materialization;

pub use lint_materialization::LintMaterialization;
pub use policy_materialization::PolicyMaterialization;

use nomos_analysis::{Context, MaterializedFact, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, ProviderId, RuleId};
use nomos_lang_rust::{FactContext, Materialization};
use nomos_platform::ProcessLauncher;
use nomos_rules::SourceFile;

use crate::composition::Recognized_Syntax_Provider;

use std::path::Path;

/// Produces one syntax fact per source and returns how many were written.
///
/// A file neither provider recognizes, or one its own recognized provider refuses to
/// parse, materializes nothing and is not dropped silently: the count returned is the
/// denominator the report prints beside the file count, and the rule's own unread-subject
/// finding names each one individually. Two independent readings of one file -- this
/// provider's and `nomos-rules`' own universe parser -- can refuse independently, and the
/// run is entitled to see which.
///
/// Dispatches per [`Recognized_Syntax_Provider`], `OD-CAPABILITY-009`'s write-side half:
/// `crate::run`'s own enrichment step narrows the read side to the same provider identity
/// through the identical function, and the two must agree for `Key_From` to ever find what
/// this function wrote.
pub fn Materialize_Syntax(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> usize
{
    let rust_production = Rust_Production(context);
    let go_production = Go_Production(context);
    let mut written = 0_usize;

    for source in sources
    {
        let Some(fact) = Materialized_Syntax_Fact(source, rust_production, go_production)
        else
        {
            continue;
        };

        // No dependency edges: a syntax fact is a leaf, read from one file's bytes and
        // from nothing this store holds.
        if store.Materialize(*fact, &[]).is_ok()
        {
            written = written.saturating_add(1);
        }
    }

    return written;
}

/// The reading context as `nomos_lang_go`'s own provider takes it -- the identical fields
/// [`Rust_Production`] already builds for `nomos_lang_rust`'s own distinct `FactContext`
/// type, restated as `nomos_lang_go`'s.
fn Go_Production(context: &Context) -> nomos_lang_go::FactContext
{
    return nomos_lang_go::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// One source's syntax fact, from whichever provider [`Recognized_Syntax_Provider`] says
/// `source.path` belongs to -- `None` for a path neither recognizes or a recognized path
/// its own provider could not parse.
///
/// Dispatches on [`Recognized_Syntax_Provider`] rather than recomputing `Recognition::Of_Path`
/// itself, the identical function `crate::run`'s own read-side enrichment calls to populate
/// `nomos_rules::SourceFile::preferred_syntax_provider` -- `OD-CAPABILITY-009`'s corrected
/// fix, one test both sides consult so they cannot independently drift.
fn Materialized_Syntax_Fact(
    source: &SourceFile,
    rust_production: FactContext,
    go_production: nomos_lang_go::FactContext,
) -> Option<Box<MaterializedFact>>
{
    let provider = Recognized_Syntax_Provider(&source.path)?;

    if provider == ProviderId::New(nomos_lang_rust::PROVIDER)
    {
        return Rust_Syntax_Fact(source, rust_production);
    }

    if provider == ProviderId::New(nomos_lang_go::PROVIDER)
    {
        return Go_Syntax_Fact(source, go_production);
    }

    return None;
}

/// `source`'s syntax fact from `nomos_lang_rust`'s own provider, or `None` if it refused to
/// parse.
fn Rust_Syntax_Fact(source: &SourceFile, rust_production: FactContext) -> Option<Box<MaterializedFact>>
{
    let Materialization::Materialized(fact) = nomos_lang_rust::Materialize(source.subject, &source.text, rust_production)
    else
    {
        return None;
    };

    return Some(fact);
}

/// `source`'s syntax fact from `nomos_lang_go`'s own provider, or `None` if it refused to
/// parse.
fn Go_Syntax_Fact(source: &SourceFile, go_production: nomos_lang_go::FactContext) -> Option<Box<MaterializedFact>>
{
    let nomos_lang_go::Materialization::Materialized(fact) = nomos_lang_go::Materialize(source.subject, &source.text, go_production)
    else
    {
        return None;
    };

    return Some(fact);
}

/// Produces one `nomos.cap.controlflow.reachability` fact per source and returns how many
/// were written.
///
/// The identical shape [`Materialize_Syntax`] has: pure, per-source, no I/O --
/// `nomos_lang_rust::reachability::Materialize` takes only a subject, source text and this
/// same [`FactContext`], the tier-1 heuristic provider `P13-CONTROLFLOW-REACHABILITY-
/// CAPABILITY` built. Not folded into `Materialize_Syntax` itself: the two are independent
/// capabilities read from the same bytes, and a shared materialization step would hide
/// which one a caller is actually asking about, the same reason [`Materialize_Dependencies`]
/// stayed a second, separate step rather than a generalization of the first.
pub fn Materialize_Reachability(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> usize
{
    let production = Rust_Production(context);
    let mut written = 0_usize;

    for source in sources
    {
        let Materialization::Materialized(fact) =
            nomos_lang_rust::reachability::Materialize(source.subject, &source.text, production)
        else
        {
            continue;
        };

        // No dependency edges: a reachability fact is a leaf, the identical reasoning
        // Materialize_Syntax gives for its own fact.
        if store.Materialize(*fact, &[]).is_ok()
        {
            written = written.saturating_add(1);
        }
    }

    return written;
}

/// Runs `cargo metadata` over `root` through `launcher`, materializes one
/// `nomos.cap.dependency.edges` fact per workspace member, and returns the subjects a rule
/// can judge them under.
///
/// A second, independent materialization step beside [`Materialize_Syntax`] rather than a
/// generalization of it: `dependency.edges` has exactly one consumer today and nothing to
/// share or schedule against `syntax.items`'s own materialization, so composing this as one
/// more hardcoded step is `OD-HOST-004`'s "composition, not choice" again, not a case for
/// the shared demand planner `ARC-ROADMAP-001` still leaves for later.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero dependency findings" either, which is
/// exactly the vacuity [`Materialize_Syntax`]'s own `NoFacts` case exists to catch one layer
/// over. So a failed materialization returns no dependency sources and one synthetic
/// finding reporting why, rather than nothing at all.
pub fn Materialize_Dependencies<P: ProcessLauncher>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &P,
) -> DependencyMaterialization
{
    let production = Cargo_Production(context);

    let facts = match nomos_lang_rust_cargo::Materialize_Workspace(root, production, launcher)
    {
        Ok(facts) => facts,
        Err(error) => return DependencyMaterialization {
            sources: Vec::new(),
            findings: vec![Dependency_Capability_Unavailable(&error)],
        },
    };

    let sources = Materialized_Dependency_Sources(facts, store);

    return DependencyMaterialization { sources, findings: Vec::new() };
}

/// What materializing `dependency.edges` facts produced: the sources a rule can judge them
/// under, and any finding the materialization itself already raised (a failed `cargo
/// metadata` call, reported rather than judged) -- named rather than left as a positional
/// pair, so a caller reads which is which without re-deriving it from
/// [`Materialize_Dependencies`]'s own body.
pub struct DependencyMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}

/// The reading context as `nomos_lang_rust_cargo`'s provider takes it.
fn Cargo_Production(context: &Context) -> nomos_lang_rust_cargo::FactContext
{
    return nomos_lang_rust_cargo::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Every `dependency.edges` fact the store accepted, as the source list `Check_Dependency_
/// Direction` can judge -- one per workspace member `store.Materialize` did not refuse.
fn Materialized_Dependency_Sources(
    facts: Vec<nomos_lang_rust_cargo::PackageFact>,
    store: &mut MemoryFactStore,
) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for package in facts
    {
        if store.Materialize(package.fact, &[]).is_ok()
        {
            let source = SourceFile::New(package.path, package.subject, String::new());
            sources.push(source);
        }
    }

    return sources;
}

/// Runs `cargo clippy --workspace --message-format=json` over `root` through `launcher`,
/// materializes one `nomos.cap.lint.diagnostics` fact per workspace member, and returns
/// the subjects a rule can judge them under -- the identical shape [`Materialize_Dependencies`]
/// already has for the identical reason: this workspace's second provider with I/O of its
/// own.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero lint findings" either, the same
/// `NoFacts`-shaped vacuity [`Materialize_Syntax`]'s own case exists to catch one layer
/// over. So a failed materialization returns no lint sources and one synthetic finding
/// reporting why, rather than nothing at all.
pub fn Materialize_Lint<P: ProcessLauncher>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &P,
) -> LintMaterialization
{
    let production = Clippy_Production(context);

    let facts = match nomos_lang_rust_clippy::Materialize_Workspace(root, production, launcher)
    {
        Ok(facts) => facts,
        Err(error) => return LintMaterialization {
            sources: Vec::new(),
            findings: vec![Lint_Capability_Unavailable(&error)],
        },
    };

    let sources = Materialized_Lint_Sources(facts, store);

    return LintMaterialization { sources, findings: Vec::new() };
}

/// The reading context as `nomos_lang_rust_clippy`'s provider takes it.
fn Clippy_Production(context: &Context) -> nomos_lang_rust_clippy::FactContext
{
    return nomos_lang_rust_clippy::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Every `lint.diagnostics` fact the store accepted, as the source list `Check_Lint_
/// Diagnostics` can judge -- one per workspace member `store.Materialize` did not refuse.
fn Materialized_Lint_Sources(
    facts: Vec<nomos_lang_rust_clippy::DiagnosticsFact>,
    store: &mut MemoryFactStore,
) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for member in facts
    {
        if store.Materialize(member.fact, &[]).is_ok()
        {
            let source = SourceFile::New(member.path, member.subject, String::new());
            sources.push(source);
        }
    }

    return sources;
}

/// The one finding a failed [`nomos_lang_rust_clippy::Materialize_Workspace`] call
/// produces -- the identical shape [`Dependency_Capability_Unavailable`] already has for
/// the identical reason: attributed to the whole tree, since a `cargo clippy` failure is
/// not about any subject this run walked.
fn Lint_Capability_Unavailable(error: &nomos_lang_rust_clippy::ClippyError) -> Finding
{
    return Finding {
        rule: RuleId::New(nomos_rules::LINT_DIAGNOSTICS),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the lint-diagnostics capability could not be materialized, so lint \
             diagnostics were not judged for anything in this run: {error}"
        ),
        locations: Vec::new(),
    };
}

/// Runs `cargo deny check bans licenses sources` over `root` through `launcher`,
/// materializes the one `nomos.cap.dependency.policy` fact this capability's
/// `IncrementalGranularity::WholeWorkspace` ceiling allows, and returns the subject a rule
/// can judge it under -- the identical shape [`Materialize_Lint`] already has for the
/// identical reason: this workspace's third provider with I/O of its own.
///
/// A failure here does not abort the run, the identical reasoning [`Materialize_Lint`]'s
/// own doc gives one layer up: a failed materialization returns no policy sources and one
/// synthetic finding reporting why, rather than nothing at all.
pub fn Materialize_Policy<P: ProcessLauncher>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    launcher: &P,
) -> PolicyMaterialization
{
    let production = Deny_Production(context);

    let fact = match nomos_lang_rust_deny::Materialize_Workspace(root, production, launcher)
    {
        Ok(fact) => fact,
        Err(error) => return PolicyMaterialization {
            sources: Vec::new(),
            findings: vec![Policy_Capability_Unavailable(&error)],
        },
    };

    let sources = Materialized_Policy_Sources(fact, store);

    return PolicyMaterialization { sources, findings: Vec::new() };
}

/// The reading context as `nomos_lang_rust_deny`'s provider takes it.
fn Deny_Production(context: &Context) -> nomos_lang_rust_deny::FactContext
{
    return nomos_lang_rust_deny::FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// The one `dependency.policy` fact as the (at most one-element) source list `Check_
/// Dependency_Policy` can judge -- empty if `store.Materialize` refused it, one entry
/// otherwise, the same shape [`Materialized_Lint_Sources`] has for a list rather than a
/// single value: `crate::run::Judged` calls every rule uniformly over a source list, and a
/// whole-workspace capability's own list just never holds more than one.
fn Materialized_Policy_Sources(fact: nomos_lang_rust_deny::PolicyFact, store: &mut MemoryFactStore) -> Vec<SourceFile>
{
    if store.Materialize(fact.fact, &[]).is_err()
    {
        return Vec::new();
    }

    return vec![SourceFile::New("workspace", fact.subject, String::new())];
}

/// The one finding a failed [`nomos_lang_rust_deny::Materialize_Workspace`] call produces
/// -- the identical shape [`Lint_Capability_Unavailable`] already has for the identical
/// reason: attributed to the whole tree, since a `cargo deny` failure is not about any
/// subject this run walked.
fn Policy_Capability_Unavailable(error: &nomos_lang_rust_deny::DenyError) -> Finding
{
    return Finding {
        rule: RuleId::New(nomos_rules::DEPENDENCY_POLICY),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the dependency-policy capability could not be materialized, so the workspace's \
             own bans/licenses/sources verdict was not judged in this run: {error}"
        ),
        locations: Vec::new(),
    };
}

/// The one finding a failed [`nomos_lang_rust_cargo::Materialize_Workspace`] call produces.
///
/// Attributed to the whole tree (`Subject_Of_Path("")`, the root's own subject per
/// `nomos_model::path`'s convention) rather than to any one file, because a `cargo metadata`
/// failure is not about any subject this run walked -- it is about whether the dependency
/// capability could answer at all. [`Applicability::ProviderUnavailable`] because the
/// provider is registered and offered; it ran and did not answer, which is exactly that
/// variant's own distinction from `MissingCapability`.
fn Dependency_Capability_Unavailable(error: &nomos_lang_rust_cargo::MetadataError) -> Finding
{
    return Finding {
        rule: RuleId::New(nomos_rules::DEPENDENCY_DIRECTION),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the dependency-edges capability could not be materialized, so dependency \
             direction was not judged for anything in this run: {error}"
        ),
        locations: Vec::new(),
    };
}

/// The reading context as `nomos_lang_rust`'s own provider takes it.
fn Rust_Production(context: &Context) -> FactContext
{
    return FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}
