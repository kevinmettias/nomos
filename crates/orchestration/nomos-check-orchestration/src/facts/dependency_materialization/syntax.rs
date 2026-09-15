//! Syntax and reachability facts, materialized per source from already-walked text.

use nomos_analysis::{Context, FactKey, FactStore, GuaranteeDigest, InputDigest, MaterializedFact, MemoryFactStore};
use nomos_contracts::ProviderId;
use nomos_lang_rust::{FactContext, Materialization};
use nomos_rules::SourceFile;

use crate::composition::Recognized_Syntax_Provider;

/// Produces one syntax fact per source and returns how many sources now have a current
/// one -- whether this call wrote it or [`Already_Current`] found the store already held
/// it. `crate::run_context::Materialized_Syntax_Facts`'s own vacuity gate and
/// `Examined::facts`'s own coverage statistic both read this as "how many of `sources`
/// this run can judge," and neither means "how much work this call actually did" --
/// [`nomos_analysis::MemoryFactStore::Materializations`]'s own delta across two calls is
/// the honest answer to that question, and what `P40-INCREMENTAL-SKIP-UNCHANGED-
/// SUBJECTS`'s own test now reads instead (`P40-INCREMENTAL-SKIP-COVERAGE-REGRESSION`
/// fixed this return value back to the coverage meaning every real caller already
/// depended on, after briefly repurposing it to mean the newly-written count with nothing
/// downstream updated to match).
///
/// A file neither provider recognizes, or one its own recognized provider refuses to
/// parse, materializes nothing and is not dropped silently: the count returned is the
/// denominator the report prints beside the file count, and the rule's own unread-subject
/// finding names each one individually. Two independent readings of one file -- this
/// provider's and `nomos-rules`' own universe parser -- can refuse independently, and the
/// run is entitled to see which.
///
/// Dispatches per [`Recognized_Syntax_Provider`], `OD-CAPABILITY-009`'s write-side half:
/// `crate::run_context`'s own enrichment step narrows the read side to the same provider identity
/// through the identical function, and the two must agree for `Key_From` to ever find what
/// this function wrote.
///
/// # Skipping a subject the store already has current
///
/// `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS`: before parsing, [`Already_Current`] builds
/// the exact [`FactKey`] parsing `source` would produce -- from `source.text`'s own raw
/// digest, never from a parse -- and asks `store` whether it already holds a live fact
/// under it. A caller that reuses the same `store` (and therefore the same, monotonically
/// advancing generation `crate::facts::Ingested_Workspace` reads off a reused `Workspace`)
/// across two calls pays for a real parse only for a subject whose own bytes moved since
/// the store last saw it -- but still counts that subject as covered either way.
pub fn Materialize_Syntax(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> usize
{
    let mut current = 0_usize;

    for source in sources
    {
        if Materialize_Source_Syntax(source, context, store)
        {
            current = current.saturating_add(1);
        }
    }

    return current;
}

/// One source's contribution to [`Materialize_Syntax`]'s own count: `true` when `source`
/// has a current `nomos.cap.syntax.items` fact once this returns -- either because
/// [`Already_Current`] found the store already held one, or because this call wrote one --
/// and `false` for a path neither provider recognizes, or for a recognized path its own
/// provider could not parse.
///
/// Extracted from [`Materialize_Syntax`]'s own loop rather than inlined there, and not a
/// promise about coverage: a `false` is still counted by the caller exactly the way
/// [`Materialize_Syntax`]'s own doc counts every uncovered source -- never dropped
/// silently, named by the rule's own unread-subject finding instead.
fn Materialize_Source_Syntax(source: &SourceFile, context: &Context, store: &mut MemoryFactStore) -> bool
{
    if Already_Current(source, context, store)
    {
        return true;
    }

    let Some(fact) = Materialized_Syntax_Fact(source, context)
    else
    {
        return false;
    };

    // No dependency edges: a syntax fact is a leaf, read from one file's bytes and from
    // nothing this store holds.
    return store.Materialize(*fact, &[]).is_ok();
}

/// Whether `store` already holds a live `nomos.cap.syntax.items` fact for `source` at
/// `context.generation` -- `false` for a path neither provider recognizes, so an
/// unrecognized source always falls through to [`Materialized_Syntax_Fact`]'s own,
/// unchanged "not dropped silently" handling.
///
/// Built from the same public pieces `nomos_lang_rust`/`nomos_lang_go`'s own provider
/// combines into a [`FactKey`] internally (`Capability`, `CONTRACT_VERSION`, `PROVIDER`,
/// `Declared_Guarantee`), restated here rather than exposed as a shared constructor: this
/// is the one caller outside either provider that ever needs to know what a fact it did
/// not produce would be keyed under, and [`InputDigest::Of`] over `source.text`'s own
/// bytes is a raw hash, not a parse -- the whole reason this check can run before one.
fn Already_Current(source: &SourceFile, context: &Context, store: &MemoryFactStore) -> bool
{
    let Some(provider) = Recognized_Syntax_Provider(&source.path)
    else
    {
        return false;
    };
    let Some(key) = Syntax_Fact_Key(source, &provider, context)
    else
    {
        return false;
    };

    return store.Current(&key.At(context.generation), context.generation).is_some();
}

/// The [`FactKey`] a real materialization of `source` through `provider` would produce --
/// `None` if `provider` is neither of the two this capability has.
fn Syntax_Fact_Key(source: &SourceFile, provider: &ProviderId, context: &Context) -> Option<FactKey>
{
    let guarantee = if *provider == ProviderId::New(nomos_lang_rust::PROVIDER)
    {
        nomos_lang_rust::Declared_Guarantee()
    }
    else if *provider == ProviderId::New(nomos_lang_go::PROVIDER)
    {
        nomos_lang_go::Declared_Guarantee()
    }
    else
    {
        return None;
    };

    return Some(FactKey {
        contract: nomos_cap_syntax::Capability(),
        contract_version: nomos_cap_syntax::CONTRACT_VERSION,
        subject: source.subject,
        semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
        provider: provider.clone(),
        provider_version: nomos_cap_syntax::CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    });
}

/// One source's syntax fact, from whichever provider [`Recognized_Syntax_Provider`] says
/// `source.path` belongs to -- `None` for a path neither recognizes or a recognized path
/// its own provider could not parse.
///
/// Dispatches on [`Recognized_Syntax_Provider`] rather than recomputing `Recognition::Of_Path`
/// itself, the identical function `crate::run_context`'s own read-side enrichment calls to populate
/// `nomos_rules::SourceFile::preferred_syntax_provider` -- `OD-CAPABILITY-009`'s corrected
/// fix, one test both sides consult so they cannot independently drift.
fn Materialized_Syntax_Fact(source: &SourceFile, context: &Context) -> Option<Box<MaterializedFact>>
{
    let provider = Recognized_Syntax_Provider(&source.path)?;

    if provider == ProviderId::New(nomos_lang_rust::PROVIDER)
    {
        return Rust_Syntax_Fact(source, Rust_Production(context));
    }

    if provider == ProviderId::New(nomos_lang_go::PROVIDER)
    {
        return Go_Syntax_Fact(source, Go_Production(context));
    }

    return None;
}

/// `source`'s syntax fact from `nomos_lang_rust`'s own provider, or `None` if it refused to
/// parse.
fn Rust_Syntax_Fact(source: &SourceFile, rust_production: FactContext) -> Option<Box<MaterializedFact>>
{
    let Materialization::Materialized(fact) = nomos_lang_rust::Materialize_Syntax_Fact(source.subject, &source.text, rust_production)
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
    let nomos_lang_go::Materialization::Materialized(fact) = nomos_lang_go::Materialize_Syntax_Fact(source.subject, &source.text, go_production)
    else
    {
        return None;
    };

    return Some(fact);
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

/// Produces one `nomos.cap.controlflow.reachability` fact per source and returns how many
/// were written.
///
/// The identical shape [`Materialize_Syntax`] has: pure, per-source, no I/O --
/// `nomos_lang_rust::reachability::Materialize_Reachability_Fact` takes only a subject, source text and this
/// same [`FactContext`], the tier-1 heuristic provider `P13-CONTROLFLOW-REACHABILITY-
/// CAPABILITY` built. Not folded into `Materialize_Syntax` itself: the two are independent
/// capabilities read from the same bytes, and a shared materialization step would hide
/// which one a caller is actually asking about, the same reason
/// [`super::Materialize_Dependencies`] stayed a second, separate step rather than a
/// generalization of the first.
pub fn Materialize_Reachability(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> usize
{
    let production = Rust_Production(context);
    let mut written = 0_usize;

    for source in sources
    {
        let Materialization::Materialized(fact) =
            nomos_lang_rust::reachability::Materialize_Reachability_Fact(source.subject, &source.text, production)
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
