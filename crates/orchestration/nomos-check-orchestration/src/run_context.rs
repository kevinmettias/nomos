//! Running one check over already-walked source, apart from finding that source or
//! rendering what came of it.
//!
//! # What stays here, and what moved out
//!
//! The entry points stay, because they are this module's public surface and the one place a
//! caller looks. Beside them sit the private types a run's three sides all share --
//! [`RunEnvironment`], [`RunState`], [`CapabilityMaterialization`],
//! [`MaterializationEnvironment`], [`JudgeEnvironment`] and [`Reassessment`] -- deliberately,
//! so that a child module reading one needs no visibility annotation of its own: an item
//! private to this module is readable in every descendant of it. The behaviour those types
//! carry moved to the submodule named for it: `capabilities` owns materializing every family
//! a selection feeds on, `judging` owns the run over what was materialized, and `tests` owns
//! the proofs that used to sit at the bottom of this file, where they had grown past the
//! size a reader can hold beside the code they exercise.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_capability::Registry;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use nomos_rules::{RequiredFact, SourceFile, DESCRIPTORS};
use nomos_workspace::{BuildVariant, Workspace};
use std::path::Path;

use crate::composed_providers::{ComposedProviders, Recognized_Language, Recognized_Syntax_Provider, SyntaxProvider};
use crate::composition::{Composed_Providers, Registered};
use crate::facts::{Ingested_Workspace, Materialize_Syntax};
use crate::CheckOutcome;
use crate::SupportingFactTrail;

mod capabilities;
mod judging;
mod rule_reassessment_cache;

pub use rule_reassessment_cache::RuleReassessmentCache;

/// [`Run`]'s build variant, its subprocess root, the launcher those subprocesses run
/// through, the filesystem a repository-declared policy capability (`nomos.cap.naming.
/// policy` and its siblings) is read through, and the workspace and fact store `Run` reads
/// and writes -- grouped into one value so [`Run`] stays within this crate's own
/// parameter-count limit. See [`Run`]'s own documentation for why each is a composition-root
/// value this crate cannot compute for itself.
pub struct RunContext<'a, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    pub variant: BuildVariant,
    pub root: &'a Path,
    pub launcher: &'a Launcher,
    pub filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from, rather than from this
    /// process's own ambient state. `OD-HOST-001`: the composition root chooses it.
    pub environment: &'a Env,
    pub workspace: &'a mut Option<Workspace>,
    pub store: &'a mut MemoryFactStore,
}

/// Composes the capability registry, ingests `sources` into one workspace state, materializes
/// a syntax fact per file, and, for each of the completeness, naming-convention,
/// dependency-direction, lint-diagnostics, dependency-policy and unread-reaches-finding
/// rules `selected` asks for (every one when `selected` is empty, the same "empty is
/// everything" default `nomos_gate_orchestration::RuleSelector::include` already has),
/// materializes the fact that rule needs and runs it over the result.
///
/// Which facts `selected` obliges this call to materialize is derived from the selection
/// rather than mapped by hand: [`capabilities::Demanded_Families`] unions `nomos_rules::
/// RuleDescriptor::requires` over the selected rules. `OD-GATE-017` accepted the hand-written
/// mapping this replaces, and `P102` measured what it cost -- the same "composition, not
/// choice" shape a fourth unconditional rule already used (`OD-HOST-004`), still extended to
/// the second axis that mapping named: whether a rule's own materialization runs at all, not
/// only whether its finding counts toward a disposition
/// `nomos_gate_orchestration::RuleSelector` narrows after the fact.
///
/// `sources` is the walk, already done -- `nomos-cli::check::sources::Walked_Sources` stayed
/// in the composition root, and its own doc carries the current reason:
/// [`nomos_platform::FileSystem`]'s `Read_Directory` is one level by `OD-PLATFORM-002`'s own
/// floor, not a traversal. `context.variant` is what that root's own binary was
/// compiled as, read through `env!` there because that macro resolves against the
/// *compiling* crate and cannot be read correctly from this one. `context.root` is the
/// tree `sources` was walked from -- carried separately because the dependency-edges,
/// lint-diagnostics and dependency-policy providers each run their own subprocess (`cargo
/// metadata`, `cargo clippy`, `cargo deny`) rather than reading bytes `sources` already
/// holds; every other provider in this workspace is a pure function over bytes a caller
/// already read. `context.launcher` is what those subprocess calls run through -- generic
/// the same way `nomos_work_orchestration::Run` is generic over [`nomos_platform`]'s
/// traits, so this crate depends on `nomos-platform` and not on any concrete implementation
/// of it; the composition root supplies one. `context.workspace` and `context.store` are the
/// caller's, not this function's own -- `OD-ANALYSIS-009`'s own first real increment. Every
/// caller this crate has today passes `&mut None` and a freshly constructed
/// [`MemoryFactStore`], which reproduces exactly what this function used to do
/// unconditionally: build both from nothing, on every call. What changes is that a caller is
/// no longer forced to. Passing the *same* `workspace` and `store` across two calls reuses
/// the workspace's own real diff ([`nomos_workspace::Workspace::Apply`] compares each
/// submitted path's content against what it already holds, so an unmoved file is
/// `Redundant` and the generation only advances when something genuinely did) and the
/// store's own generation-scoped history, instead of starting from an empty tree and an
/// empty store every time. This function still ingests every source in `sources` and still
/// runs every provider a selection demands on every call; what it no longer does is *file* a
/// fact the store is already serving -- `Materialize_Syntax` since
/// `P40-INCREMENTAL-SKIP-UNCHANGED-SUBJECTS` and every other family since
/// `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY`, both through `crate::facts::currency`. The
/// five are grouped into [`RunContext`] so this function stays within this crate's own
/// parameter-count limit.
///
/// Writes nothing and never exits: [`CheckOutcome`] is the whole answer, the same
/// division `nomos_work_orchestration::Run` draws around [`nomos_work_orchestration`]'s own
/// `WorkOutcome`. An empty `sources` is a composition root's decision
/// (`CheckOutcome::NoSource`) made before this function is ever called, not a case this
/// function classifies.
///
/// Delegates to [`Run_Reassessing`] with a cache built fresh for this call alone and
/// dropped at its end -- every rule this crate composes has never run under that cache, so
/// every one still runs, unconditionally, exactly as this function did before
/// [`RuleReassessmentCache`] existed. A caller wanting the skip keeps its own cache across
/// calls and calls [`Run_Reassessing`] directly instead.
#[must_use]
pub fn Run<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    context: RunContext<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
) -> CheckOutcome
{
    let mut reassessment = RuleReassessmentCache::New();

    return Run_Reassessing(sources, context, selected, &mut reassessment);
}

/// [`Run`], with one addition: a rule already recorded in `reassessment` does not run its
/// closure again this call when none of its own `nomos_rules::RuleDescriptor.requires`
/// names a family that changed since it was recorded -- see `rule_reassessment`'s own
/// module doc for exactly which rules that covers today, and why it stops there.
///
/// The currency check itself is not this function's own and never was: every family
/// `crate::facts` materializes proves its fact is current before filing it, on [`Run`] as
/// much as here, and `crate::facts::currency` is where that is decided.
/// `Materialize_Syntax`'s own check came first and is still the only one that runs before its
/// provider does, so a source whose bytes did not move never re-parses. What only this
/// function adds is skipping the *rule* on top of that, for any rule none of whose declared
/// families moved -- which, since
/// `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY`, is no longer just the rules reading syntax
/// alone.
#[must_use]
pub fn Run_Reassessing<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    sources: &[SourceFile],
    context: RunContext<'_, Launcher, Fs, Env>,
    selected: &[RuleId],
    reassessment: &mut RuleReassessmentCache,
) -> CheckOutcome
{
    let RunContext { variant, root, launcher, filesystem, environment, workspace, store } = context;

    let providers = Composed_Providers::<Launcher, Fs, Env>();
    let recognized = Recognized_Sources(sources, &providers.syntax);
    let sources: &[SourceFile] = &recognized;

    let composition = RunComposition { variant, workspace, store: &mut *store, syntax: &providers.syntax };
    let composed = match Composed_Run(sources, composition)
    {
        Ok(composed) => composed,
        Err(outcome) => return outcome,
    };

    let run = RunEnvironment {
        root, launcher, filesystem, environment, providers: &providers, registry: &composed.registry, context: composed.context, selected,
    };
    let mut state = RunState { store, reassessment, changed: composed.changed };
    let findings = Judged_Over(sources, run, &mut state);

    return Outcome_Of(Examination { files: sources.len(), facts: composed.facts }, findings, state.reassessment.Supporting_Facts().clone());
}

/// The registry composed, `sources` ingested and the syntax facts materialized -- the three
/// steps this crate takes before any rule is judged, composed here so [`Run_Reassessing`]'s
/// own body stays within this crate's own function-size limit and names one step instead of
/// five. [`ComposedRun::changed`] already carries the one family those three steps account
/// for (`RequiredFact::SyntaxItems`); every other family
/// [`capabilities::Materialize_Capabilities`] writes is appended after this returns.
fn Composed_Run(sources: &[SourceFile], composition: RunComposition<'_>) -> Result<ComposedRun, CheckOutcome>
{
    let (registry, context) = Composed_Registry_And_Context(sources, composition.variant, composition.workspace)?;

    let materializations_before_syntax = composition.store.Materializations();
    let Some(facts) = Materialized_Syntax_Facts(sources, &context, composition.store, composition.syntax)
    else
    {
        return Err(CheckOutcome::NoFacts { files: sources.len() });
    };

    let mut changed = Vec::new();
    if composition.store.Materializations() > materializations_before_syntax
    {
        changed.push(RequiredFact::SyntaxItems);
    }

    return Ok(ComposedRun { registry, context, facts, changed });
}

/// The build variant, the workspace and store a run reads and writes, and the composed
/// syntax offers its first materialization step goes through -- grouped into one value so
/// [`Composed_Run`] stays within this crate's own parameter-count limit.
struct RunComposition<'a>
{
    variant: BuildVariant,
    workspace: &'a mut Option<Workspace>,
    store: &'a mut MemoryFactStore,
    syntax: &'a [SyntaxProvider],
}

/// What [`Composed_Run`] assembled: the composed registry, the ingested [`Context`], the
/// count [`Materialized_Syntax_Facts`] answered with, and the one capability family those
/// steps changed -- named rather than left as a positional triple, so a caller reads which
/// is which without re-deriving it from [`Composed_Run`]'s own body.
struct ComposedRun
{
    registry: Registry,
    context: Context,
    facts: usize,
    changed: Vec<RequiredFact>,
}

/// The registry composed and `sources` ingested through it, or the [`CheckOutcome`] that
/// already answers the run when either step refuses. `workspace` is threaded through to
/// [`Ingested_Workspace`] rather than built here: reuse across two calls to [`Run`] is a
/// property of the same [`Workspace`] object seeing a second `Workspace::Apply`, not
/// something this function can arrange after the fact.
fn Composed_Registry_And_Context(
    sources: &[SourceFile], variant: BuildVariant, workspace: &mut Option<Workspace>,
) -> Result<(Registry, Context), CheckOutcome>
{
    let registry = Registered().map_err(CheckOutcome::Contradictory)?;
    let context = Ingested_Workspace(sources, &registry, variant, workspace).map_err(|_error| return CheckOutcome::Unreadable)?;

    return Ok((registry, context));
}

/// The syntax facts materialized into `store`, or `None` when there were none to judge --
/// [`Run`]'s own first early exit, given a name so its body reads as one decision per line.
fn Materialized_Syntax_Facts(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore, syntax: &[SyntaxProvider]) -> Option<usize>
{
    let facts = Materialize_Syntax(sources, context, store, syntax);
    if facts == 0
    {
        return None;
    }

    return Some(facts);
}

/// [`Run`]'s own root, launcher, filesystem, registry, composed context and rule
/// selection -- everything [`Judged_Over`] needs beside the sources and mutable state it is
/// handed separately, grouped so that function's parameter list names one environment
/// instead of six loose values.
struct RunEnvironment<'a, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    root: &'a Path,
    launcher: &'a Launcher,
    filesystem: &'a Fs,
    environment: &'a Env,
    /// Every analysis provider this run materializes through, composed once by
    /// [`Run_Reassessing`] and read by every section below it. `OD-ROADMAP-005` decision item
    /// 1: the service receives the providers rather than naming them.
    providers: &'a ComposedProviders<Launcher, Fs, Env>,
    registry: &'a Registry,
    context: Context,
    selected: &'a [RuleId],
}

/// The store, the reassessment cache, and which capability families this call has changed
/// so far -- every value [`Judged_Over`] mutates, grouped into one so that function and
/// [`Run_Reassessing`] both stay within this crate's own parameter-count limit. `changed`
/// starts already carrying whatever [`Composed_Run`] found before [`Judged_Over`] is
/// ever called (the syntax family, from its own before/after [`MemoryFactStore::
/// Materializations`] snapshot) and `capabilities::Materialize_Capabilities` appends the
/// rest to it.
struct RunState<'a>
{
    store: &'a mut MemoryFactStore,
    reassessment: &'a mut RuleReassessmentCache,
    changed: Vec<RequiredFact>,
}

/// Every capability [`Run`] can materialize, judged -- the two steps [`Run`] itself used to
/// inline, composed here so its own body names one step instead of four.
fn Judged_Over<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(sources: &[SourceFile], environment: RunEnvironment<'_, Launcher, Fs, Env>, state: &mut RunState<'_>) -> Vec<Finding>
{
    let mut materialization_environment = MaterializationEnvironment {
        root: environment.root,
        context: &environment.context,
        store: state.store,
        launcher: environment.launcher,
        filesystem: environment.filesystem,
        environment: environment.environment,
        providers: environment.providers,
    };
    let capabilities = capabilities::Materialize_Capabilities(sources, &mut materialization_environment, environment.selected, &mut state.changed);

    let judge_environment = JudgeEnvironment { store: state.store, registry: environment.registry, context: environment.context };
    let reassessment = Reassessment { selected: environment.selected, cache: state.reassessment, changed: &state.changed };
    return judging::Judged_Findings(sources, capabilities, &judge_environment, reassessment);
}

/// What [`capabilities::Materialize_Capabilities`] produced: one [`MaterializedCapability`]
/// per family a rule is judged over -- named rather than left as positional pairs, the same
/// reason [`crate::facts::DependencyMaterialization`],
/// [`crate::facts::LintMaterialization`] and [`crate::facts::PolicyMaterialization`] each
/// exist one layer under it.
struct CapabilityMaterialization
{
    dependency: MaterializedCapability,
    lint: MaterializedCapability,
    policy: MaterializedCapability,
    review: MaterializedCapability,
    /// The two compiler-backed families. They are slices of their own rather than entries in
    /// the walked sources for the same reason the four above them are: each is one fact about
    /// a project, filed under a subject no walked file carries.
    copy_clones: MaterializedCapability,
    nested_locks: MaterializedCapability,
}

/// The sources one capability family materialized, and any finding materializing it already
/// raised on its own -- the same two halves
/// [`crate::facts::DependencyMaterialization`] carries one layer down, kept as one value here
/// so that [`CapabilityMaterialization`] names four families rather than eight parallel lists
/// whose halves a reader has to pair up by their shared suffix.
struct MaterializedCapability
{
    sources: Vec<SourceFile>,
    findings: Vec<Finding>,
}

/// The `root`, `context`, `store`, `launcher`, `filesystem` and `environment` every section
/// of `capabilities::Materialize_Capabilities` reads or writes through -- grouped into one
/// value so that function takes those six as one parameter rather than six.
struct MaterializationEnvironment<'a, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    root: &'a Path,
    context: &'a Context,
    store: &'a mut MemoryFactStore,
    launcher: &'a Launcher,
    filesystem: &'a Fs,
    environment: &'a Env,
    providers: &'a ComposedProviders<Launcher, Fs, Env>,
}

/// The `store`, `registry` and `context` `judging::Judged_Findings` reads the
/// `nomos_analysis::Reader` from -- grouped into one value so that function takes those
/// three as one parameter rather than three.
struct JudgeEnvironment<'a>
{
    store: &'a MemoryFactStore,
    registry: &'a Registry,
    context: Context,
}

/// A rule's selection, its reassessment cache, and which capability families this call has
/// changed -- grouped into one value so `judging`'s functions each take it as one parameter
/// rather than three.
struct Reassessment<'a>
{
    selected: &'a [RuleId],
    cache: &'a mut RuleReassessmentCache,
    changed: &'a [RequiredFact],
}

/// How much of the world the run saw, before it is paired with what the run found.
///
/// Named rather than passed as two adjacent `usize` parameters, which is the transposition
/// `crate::examined::Examined` already exists to prevent one layer out -- and what keeps
/// [`Outcome_Of`] within this crate's own parameter-count limit now that the trail is a
/// third thing the outcome carries.
struct Examination
{
    /// Files the walk read.
    files: usize,
    /// Files a syntax fact was materialized for.
    facts: usize,
}

/// The whole run, once judging is done -- how much it examined, the claim its own findings
/// support, and which facts each rule read to reach them.
fn Outcome_Of(examination: Examination, findings: Vec<Finding>, supporting_facts: SupportingFactTrail) -> CheckOutcome
{
    use crate::examined::{Claim_Of, Examined};

    let examined = Examined { files: examination.files, facts: examination.facts };
    let claim = Claim_Of(&findings);

    return CheckOutcome::Judged { findings, examined, claim, supporting_facts };
}

/// Every rule [`Run`] composes, in the order it runs them.
///
/// This is `nomos_rules::DESCRIPTORS` read for its identifiers, and it is derived rather
/// than written down: `judging::Findings_For_Selected_Rules` walks the same list in the same
/// order, so a rule declared there appears here with no second edit and none can be declared
/// without appearing.
///
/// It was not always derived. `OD-GATE-020` measured the gate registry offering eight rules
/// against the fifty-six then composed here, found the declared parity between the two had
/// gone false silently twice, and named why closing it by hand again would not fix the
/// shape. The answer then was one array literal read from two places; the answer now is no
/// array literal at all, which `OD-RULES-027` decided was available once the seventy
/// closures the old table held were measured rather than assumed to be seventy different
/// things.
#[must_use]
pub fn Composed_Rules() -> Vec<RuleId>
{
    return DESCRIPTORS.iter().map(|descriptor| return RuleId::New(descriptor.id)).collect();
}

/// `sources`, each carrying its own resolved [`nomos_rules::SourceFile::preferred_syntax_provider`]
/// -- `OD-CAPABILITY-009`'s corrected fix, computed once here against the composed syntax
/// offers this run was given; `nomos_rules` never asks the question and this crate no longer
/// names a provider to answer it. Every rule this crate composes sees only the
/// enriched copy, so a subject's syntax provider identity is settled before any of them run,
/// the same "carried rather than derived" reasoning [`SourceFile::subject`] already states
/// for the field this one sits beside.
///
/// `pub(crate)` rather than private to [`Run`] alone: this crate's own `tests.rs` reaches
/// past `Run` into `crate::composition` and `crate::facts` directly, by design, to prove the
/// split-composition guarantee against the store rather than against `Run`'s one call shape
/// -- and a fixture built that way needs the identical enrichment `Run` gives every other
/// caller, not a second, differently-behaved copy of it.
pub(crate) fn Recognized_Sources(sources: &[SourceFile], providers: &[SyntaxProvider]) -> Vec<SourceFile>
{
    return sources
        .iter()
        .cloned()
        .map(|mut source| {
            source.preferred_syntax_provider = Recognized_Syntax_Provider(providers, &source.path).map(|provider| return provider.provider.clone());
            source.language = Recognized_Language(providers, &source.path);
            return source;
        })
        .collect();
}

/// Whether `rule` is one `selected` asks for -- every rule when `selected` is empty, the same
/// "empty is everything" default `nomos_gate_orchestration::RuleSelector::include` already
/// has.
///
/// It sits at this module's root rather than beside either caller because both sides of a run
/// ask it: `capabilities::Demanded_Families` to decide which families to materialize, and
/// `judging::Findings_For_Selected_Rules` to decide which rules to run over them. A child
/// module reads this private item directly, the same access the neighboring shared types are
/// placed here to give it.
fn Is_Rule_Selected(selected: &[RuleId], rule: &str) -> bool
{
    return selected.is_empty() || selected.iter().any(|id| return id.As_Str() == rule);
}

#[cfg(test)]
mod tests;
