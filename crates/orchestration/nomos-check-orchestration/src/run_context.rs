//! Running one check over already-walked source, apart from finding that source or
//! rendering what came of it.

use nomos_analysis::{Context, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::ProcessLauncher;
use nomos_rules::{
    Check_A_Credential_Is_Not_Hardcoded_In_Source, Check_A_Discarded_Error_Is_Explained,
    Check_A_Package_Is_Named_After_Its_Directory, Check_A_Rust_Path_Stays_Within_Its_Own_Subtree,
    Check_A_Script_Declares_Its_Purpose, Check_A_Secret_Does_Not_Travel_In_A_Url, Check_A_Skipped_Test_States_Why,
    Check_An_Excluded_File_Says_Why, Check_Atomic_Ordering_Choices_Are_Justified,
    Check_Certificate_Verification_Is_Not_Disabled, Check_Completeness_Mirrors, Check_Cross_Language_Correspondence,
    Check_Data_Names_Stay_Lower_Snake, Check_Declared_Tooling_Language_For_Scripts, Check_Dependency_Direction,
    Check_Dependency_Policy, Check_Deprecation_Carries_A_Reason, Check_Eager_Vs_Lazy_Context,
    Check_Error_Message_Has_No_Trailing_Punctuation, Check_Error_Message_Starts_Lowercase,
    Check_Every_Allow_Carries_A_Justification, Check_Every_Member_Declares_A_Band,
    Check_Exported_Go_Functions_Use_Upper_Snake_Case, Check_File_Name_Matches_Declared_Type,
    Check_File_Size_Justification_Trigger, Check_Go_Constants_Split_By_Export, Check_Go_File_Size_Hard_Trigger,
    Check_Go_File_Size_Review_Trigger, Check_Go_Helpers_Package_Five_Inputs, Check_Go_Type_Names_Use_Camel_Case,
    Check_Go_Variables_Use_Lower_Snake_Case, Check_Lint_Diagnostics, Check_Naming_Convention, Check_No_Mod_Rs_Files,
    Check_No_Trailing_Whitespace, Check_Parameter_Count, Check_Relaxed_Not_Used_When_Ordering_Matters,
    Check_Scripts_Use_A_Portable_Shebang, Check_Seqcst_Justified_Explicitly, Check_Shared_Interior_Mutability_Says_Why,
    Check_Suppression_Directives_Carry_A_Reason, Check_Todo_Format,
    Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter, Check_Unread_Reaches_A_Finding,
    Check_Unsafe_Justification, Check_Workspace_Markers_Carry_A_Reason, SourceFile,
    A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE, A_DISCARDED_ERROR_IS_EXPLAINED, A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY,
    A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, A_SCRIPT_DECLARES_ITS_PURPOSE, A_SECRET_DOES_NOT_TRAVEL_IN_A_URL,
    A_SKIPPED_TEST_STATES_WHY, AN_EXCLUDED_FILE_SAYS_WHY, ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED,
    CERTIFICATE_VERIFICATION_IS_NOT_DISABLED, COMPLETENESS_MIRROR, CONSTANTS_SPLIT_BY_EXPORT,
    CROSS_LANGUAGE_CORRESPONDENCE, DATA_NAMES_STAY_LOWER_SNAKE, DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS,
    DEPENDENCY_COMPLETENESS, DEPENDENCY_DIRECTION, DEPENDENCY_POLICY, DEPRECATION, EAGER_VS_LAZY_CONTEXT,
    EVERY_ALLOW_CARRIES_A_JUSTIFICATION, EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, FILE_NAME_MATCHES_DECLARED_TYPE,
    FILE_SIZE_JUSTIFICATION_TRIGGER, FIVE_HUNDRED_LINE_REVIEW_TRIGGER,
    GO_HELPERS_PACKAGE_FIVE_INPUTS, GO_VARIABLES_USE_LOWER_SNAKE_CASE, LINT_DIAGNOSTICS, LOWERCASE_FIRST_LETTER,
    NAMING_CONVENTION, NO_MOD_RS_FILES, NO_TRAILING_PUNCTUATION, NO_TRAILING_WHITESPACE,
    ONE_THOUSAND_LINE_HARD_TRIGGER, PARAMETER_COUNT,
    RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, SCRIPTS_USE_A_PORTABLE_SHEBANG, SEQCST_JUSTIFIED_EXPLICITLY,
    SHARED_INTERIOR_MUTABILITY_SAYS_WHY, SUPPRESSION_DIRECTIVES_CARRY_A_REASON,
    TODO_FORMAT, TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE, UNREAD_REACHES_FINDING,
    UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, UNSAFE_JUSTIFICATION, WORKSPACE_MARKERS_CARRY_A_REASON,
};
use nomos_workspace::BuildVariant;
use std::path::Path;

use crate::composition::{Recognized_Syntax_Provider, Registered};
use crate::facts::{
    DependencyMaterialization, Ingested_Workspace, LintMaterialization, Materialize_Dependencies, Materialize_Lint,
    Materialize_Policy, Materialize_Reachability, Materialize_Syntax, PolicyMaterialization,
};
use crate::CheckOutcome;

/// How many rules [`Rule_Findings`] runs -- authoritative at module scope because the array
/// literal it sizes is the one and only place this count is spent.
const RULE_COUNT: usize = 46;

/// [`Run`]'s build variant, its subprocess root, and the launcher those subprocesses run
/// through -- grouped into one value so [`Run`] stays within this crate's own
/// parameter-count limit. See [`Run`]'s own documentation for why each is a
/// composition-root value this crate cannot compute for itself.
pub struct RunContext<'a, Launcher: ProcessLauncher>
{
    pub variant: BuildVariant,
    pub root: &'a Path,
    pub launcher: &'a Launcher,
}

/// Composes the capability registry, ingests `sources` into one workspace state, materializes
/// a syntax fact per file, and, for each of the completeness, naming-convention,
/// dependency-direction, lint-diagnostics, dependency-policy and unread-reaches-finding
/// rules `selected` asks for (every one when `selected` is empty, the same "empty is
/// everything" default `nomos_gate_orchestration::RuleSelector::include` already has),
/// materializes the fact that rule needs and runs it over the result.
///
/// `selected` is a fixed, hand-written mapping from [`RuleId`] to the fact(s) it needs, per
/// `OD-GATE-017`: the same "composition, not choice" shape a fourth unconditional rule already
/// used (`OD-HOST-004`), extended to a second axis -- whether a rule's own materialization
/// runs at all, not only whether its finding counts toward a disposition
/// `nomos_gate_orchestration::RuleSelector` narrows after the fact.
///
/// `sources` is the walk, already done -- this crate has no [`nomos_platform::FileSystem`]
/// port to walk a directory through, the same reason `nomos-cli::check::sources::Walked`
/// stayed in the composition root. `context.variant` is what that root's own binary was
/// compiled as, read through `env!` there because that macro resolves against the
/// *compiling* crate and cannot be read correctly from this one. `context.root` is the
/// tree `sources` was walked from -- carried separately because the dependency-edges,
/// lint-diagnostics and dependency-policy providers each run their own subprocess (`cargo
/// metadata`, `cargo clippy`, `cargo deny`) rather than reading bytes `sources` already
/// holds; every other provider in this workspace is a pure function over bytes a caller
/// already read. `context.launcher` is what those subprocess calls run through -- generic
/// the same way `nomos_work_orchestration::Run` is generic over [`nomos_platform`]'s
/// traits, so this crate depends on `nomos-platform` and not on any concrete implementation
/// of it; the composition root supplies one. The three are grouped into [`RunContext`] so
/// this function stays within this crate's own parameter-count limit.
///
/// Writes nothing and never exits: [`CheckOutcome`] is the whole answer, the same
/// division `nomos_work_orchestration::Run` draws around [`nomos_work_orchestration`]'s own
/// `WorkOutcome`. An empty `sources` is a composition root's decision
/// (`CheckOutcome::NoSource`) made before this function is ever called, not a case this
/// function classifies.
#[must_use]
pub fn Run<Launcher: ProcessLauncher>(
    sources: &[SourceFile],
    context: RunContext<'_, Launcher>,
    selected: &[RuleId],
) -> CheckOutcome
{
    let RunContext { variant, root, launcher } = context;

    let recognized = Recognized_Sources(sources);
    let sources: &[SourceFile] = &recognized;

    let (registry, context) = match Composed_Registry_And_Context(sources, variant)
    {
        Ok(composed) => composed,
        Err(outcome) => return outcome,
    };

    let mut store = MemoryFactStore::New();
    let facts = match Materialized_Syntax_Facts(sources, &context, &mut store)
    {
        Some(facts) => facts,
        None => return CheckOutcome::NoFacts { files: sources.len() },
    };

    let environment = RunEnvironment { root, launcher, registry: &registry, context };
    let findings = Judged_Over(sources, environment, &mut store, selected);

    return Outcome_Of(sources.len(), facts, findings);
}

/// The registry composed and `sources` ingested through it, or the [`CheckOutcome`] that
/// already answers the run when either step refuses.
fn Composed_Registry_And_Context(sources: &[SourceFile], variant: BuildVariant) -> Result<(Registry, Context), CheckOutcome>
{
    let registry = Registered().map_err(CheckOutcome::Contradictory)?;
    let context = Ingested_Workspace(sources, &registry, variant).map_err(|_error| return CheckOutcome::Unreadable)?;

    return Ok((registry, context));
}

/// The syntax facts materialized into `store`, or `None` when there were none to judge --
/// [`Run`]'s own first early exit, given a name so its body reads as one decision per line.
fn Materialized_Syntax_Facts(sources: &[SourceFile], context: &Context, store: &mut MemoryFactStore) -> Option<usize>
{
    let facts = Materialize_Syntax(sources, context, store);
    if facts == 0
    {
        return None;
    }

    return Some(facts);
}

/// [`Run`]'s own root, launcher, registry and composed context -- everything [`Judged_Over`]
/// needs beside the sources and store it is handed separately, grouped so that function's
/// parameter list names one environment instead of four loose values.
struct RunEnvironment<'a, Launcher: ProcessLauncher>
{
    root: &'a Path,
    launcher: &'a Launcher,
    registry: &'a Registry,
    context: Context,
}

/// Every capability [`Run`] can materialize, judged -- the two steps [`Run`] itself used to
/// inline, composed here so its own body names one step instead of four.
fn Judged_Over<Launcher: ProcessLauncher>(
    sources: &[SourceFile],
    environment: RunEnvironment<'_, Launcher>,
    store: &mut MemoryFactStore,
    selected: &[RuleId],
) -> Vec<Finding>
{
    let mut materialization_environment = MaterializationEnvironment {
        root: environment.root,
        context: &environment.context,
        store,
        launcher: environment.launcher,
    };
    let capabilities = Materialize_Capabilities(sources, &mut materialization_environment, selected);

    let judge_environment = JudgeEnvironment { store, registry: environment.registry, context: environment.context };
    return Judged_Findings(sources, capabilities, judge_environment, selected);
}

/// The dependency-edges, lint-diagnostics, dependency-policy and reachability facts,
/// materialized into `store` alongside the syntax facts [`Run`] already wrote -- the
/// capabilities beside `syntax.items` that this crate's registration composes, each with
/// its own materialization step for the reasons [`Materialize_Dependencies`],
/// [`Materialize_Lint`], [`Materialize_Policy`] and [`Materialize_Reachability`] give.
///
/// Each runs only when `selected` asks for a rule it feeds -- `DEPENDENCY_DIRECTION` or
/// `DEPENDENCY_COMPLETENESS` for the first (both judge the same `dependencies.sources`),
/// `LINT_DIAGNOSTICS` for the second, `DEPENDENCY_POLICY` for the third,
/// `UNREAD_REACHES_FINDING` for the fourth, per `OD-GATE-017`. Skipping
/// `Materialize_Dependencies`, `Materialize_Lint` or `Materialize_Policy` skips its own
/// subprocess launch entirely, not merely its finding's place in a later disposition.
fn Materialize_Capabilities<Launcher: ProcessLauncher>(
    sources: &[SourceFile],
    env: &mut MaterializationEnvironment<'_, Launcher>,
    selected: &[RuleId],
) -> CapabilityMaterialization
{
    let dependencies = Materialize_Dependency_Section(env, selected);
    let lint = Materialize_Lint_Section(env, selected);
    let policy = Materialize_Policy_Section(env, selected);
    Materialize_Reachability_Section(sources, env, selected);

    return Capability_Materialization_Of(dependencies, lint, policy);
}

/// The `root`, `context`, `store` and `launcher` every [`Materialize_Capabilities`] section
/// reads or writes through -- grouped into one value so that function takes those four as
/// one parameter rather than four.
struct MaterializationEnvironment<'a, Launcher: ProcessLauncher>
{
    root: &'a Path,
    context: &'a Context,
    store: &'a mut MemoryFactStore,
    launcher: &'a Launcher,
}

/// The dependency-edges section: [`Materialize_Dependencies`] when `selected` feeds on it,
/// an empty result otherwise.
fn Materialize_Dependency_Section<Launcher: ProcessLauncher>(
    env: &mut MaterializationEnvironment<'_, Launcher>,
    selected: &[RuleId],
) -> DependencyMaterialization
{
    if Is_Rule_Selected(selected, DEPENDENCY_DIRECTION) || Is_Rule_Selected(selected, DEPENDENCY_COMPLETENESS)
    {
        return Materialize_Dependencies(env.root, env.context, env.store, env.launcher);
    }

    return DependencyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The lint-diagnostics section: [`Materialize_Lint`] when `selected` feeds on it, an empty
/// result otherwise.
fn Materialize_Lint_Section<Launcher: ProcessLauncher>(
    env: &mut MaterializationEnvironment<'_, Launcher>,
    selected: &[RuleId],
) -> LintMaterialization
{
    if Is_Rule_Selected(selected, LINT_DIAGNOSTICS)
    {
        return Materialize_Lint(env.root, env.context, env.store, env.launcher);
    }

    return LintMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The dependency-policy section: [`Materialize_Policy`] when `selected` feeds on it, an
/// empty result otherwise.
fn Materialize_Policy_Section<Launcher: ProcessLauncher>(
    env: &mut MaterializationEnvironment<'_, Launcher>,
    selected: &[RuleId],
) -> PolicyMaterialization
{
    if Is_Rule_Selected(selected, DEPENDENCY_POLICY)
    {
        return Materialize_Policy(env.root, env.context, env.store, env.launcher);
    }

    return PolicyMaterialization { sources: Vec::new(), findings: Vec::new() };
}

/// The reachability section: [`Materialize_Reachability`] when `selected` feeds on it --
/// writes into `env.store` directly and produces no return value of its own, the same shape
/// the call it wraps already has.
fn Materialize_Reachability_Section<Launcher: ProcessLauncher>(
    sources: &[SourceFile],
    env: &mut MaterializationEnvironment<'_, Launcher>,
    selected: &[RuleId],
)
{
    if Is_Rule_Selected(selected, UNREAD_REACHES_FINDING)
    {
        Materialize_Reachability(sources, env.context, env.store);
    }
}

/// The assembly section: what the three source-and-finding materializations produced,
/// gathered into one [`CapabilityMaterialization`].
fn Capability_Materialization_Of(
    dependencies: DependencyMaterialization,
    lint: LintMaterialization,
    policy: PolicyMaterialization,
) -> CapabilityMaterialization
{
    return CapabilityMaterialization {
        dependency_sources: dependencies.sources,
        dependency_findings: dependencies.findings,
        lint_sources: lint.sources,
        lint_findings: lint.findings,
        policy_sources: policy.sources,
        policy_findings: policy.findings,
    };
}

/// What [`Materialize_Capabilities`] produced: the dependency-edges, lint-diagnostics and
/// dependency-policy sources a rule can judge, and any finding materializing one already
/// raised on its own -- named rather than left as positional pairs, the same reason
/// [`crate::facts::DependencyMaterialization`], [`crate::facts::LintMaterialization`] and
/// [`crate::facts::PolicyMaterialization`] each exist one layer under it.
struct CapabilityMaterialization
{
    dependency_sources: Vec<SourceFile>,
    dependency_findings: Vec<Finding>,
    lint_sources: Vec<SourceFile>,
    lint_findings: Vec<Finding>,
    policy_sources: Vec<SourceFile>,
    policy_findings: Vec<Finding>,
}

/// Every finding [`Rule_Findings`] produces over `sources` and `capabilities`' own source
/// lists, plus whatever [`Materialize_Capabilities`] already found on its own via
/// [`Capability_Findings`] (a failed dependency or lint materialization, reported rather
/// than judged) -- unconditionally, since each such finding already carries its own rule
/// and a caller that did not select it would never have triggered the materialization
/// that raises it.
fn Judged_Findings(sources: &[SourceFile], capabilities: CapabilityMaterialization, env: JudgeEnvironment<'_>, selected: &[RuleId]) -> Vec<Finding>
{
    let mut reader = Reader::On(env.store, env.registry, env.context);

    let mut findings = Rule_Findings(sources, &capabilities, &mut reader, selected);
    findings.extend(Capability_Findings(capabilities));

    return findings;
}

/// The `store`, `registry` and `context` [`Judged_Findings`] reads the [`Reader`] from --
/// grouped into one value so that function takes those three as one parameter rather than
/// three.
struct JudgeEnvironment<'a>
{
    store: &'a MemoryFactStore,
    registry: &'a Registry,
    context: Context,
}

/// Every finding the completeness, naming-convention, dependency-direction,
/// dependency-completeness, lint-diagnostics, dependency-policy, unread-reaches-finding,
/// cross-language-correspondence and ten text-only rules `selected` asks for produce over
/// `sources` and `capabilities`' own source lists.
///
/// The ten text-only rules (no-trailing-whitespace through no-mod-rs-files below) take only
/// `sources`, the same shape [`Check_Naming_Convention`] and every fact-reading rule above
/// it does not: a rule implemented in `nomos-rules` but never composed here reports as
/// unenforced when it is not, so wiring one in is this crate's own territory, the same
/// "composed into nomos-check-orchestration::Run" section every one of these rules' own
/// module docs already names. Three siblings this crate also implements --
/// `no-decorative-section-dividers`, `unwrap-expect-discipline` and `panics-are-justified-
/// documented-and-validated` -- are deliberately absent: this repository's own tree
/// currently violates all three (71 findings, checked by running the ten below through a
/// real `gate run --root .` first), and wiring a rule this tree fails is a different, larger
/// change than composing one it already satisfies.
fn Rule_Findings(
    sources: &[SourceFile],
    capabilities: &CapabilityMaterialization,
    reader: &mut Reader<'_, '_>,
    selected: &[RuleId],
) -> Vec<Finding>
{
    // Boxed as `dyn Fn` because the closures below close over different captures
    // (`sources`, `capabilities.dependency_sources`, `capabilities.lint_sources`, ...) and so
    // are distinct anonymous types -- an array needs one common element type, and `dyn
    // Fn` is that common type where `impl Fn` cannot be.
    let rules: [(&str, &dyn Fn(&mut Reader<'_, '_>) -> Vec<Finding>); RULE_COUNT] = [
        (COMPLETENESS_MIRROR, &|reader| return Check_Completeness_Mirrors(sources, reader)),
        (NAMING_CONVENTION, &|reader| return Check_Naming_Convention(sources, reader)),
        (DEPENDENCY_DIRECTION, &|reader| return Check_Dependency_Direction(&capabilities.dependency_sources, reader)),
        (DEPENDENCY_COMPLETENESS, &|reader| return Check_Every_Member_Declares_A_Band(&capabilities.dependency_sources, reader)),
        (LINT_DIAGNOSTICS, &|reader| return Check_Lint_Diagnostics(&capabilities.lint_sources, reader)),
        (DEPENDENCY_POLICY, &|reader| return Check_Dependency_Policy(&capabilities.policy_sources, reader)),
        (UNREAD_REACHES_FINDING, &|reader| return Check_Unread_Reaches_A_Finding(sources, reader)),
        (CROSS_LANGUAGE_CORRESPONDENCE, &|reader| return Check_Cross_Language_Correspondence(sources, reader)),
        (NO_TRAILING_WHITESPACE, &|_reader| return Check_No_Trailing_Whitespace(sources)),
        (TODO_FORMAT, &|_reader| return Check_Todo_Format(sources)),
        (DEPRECATION, &|_reader| return Check_Deprecation_Carries_A_Reason(sources)),
        (A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, &|_reader| return Check_A_Rust_Path_Stays_Within_Its_Own_Subtree(sources)),
        (SHARED_INTERIOR_MUTABILITY_SAYS_WHY, &|_reader| return Check_Shared_Interior_Mutability_Says_Why(sources)),
        (EVERY_ALLOW_CARRIES_A_JUSTIFICATION, &|_reader| return Check_Every_Allow_Carries_A_Justification(sources)),
        (UNSAFE_JUSTIFICATION, &|_reader| return Check_Unsafe_Justification(sources)),
        (SCRIPTS_USE_A_PORTABLE_SHEBANG, &|_reader| return Check_Scripts_Use_A_Portable_Shebang(sources)),
        (A_SCRIPT_DECLARES_ITS_PURPOSE, &|_reader| return Check_A_Script_Declares_Its_Purpose(sources)),
        (NO_MOD_RS_FILES, &|_reader| return Check_No_Mod_Rs_Files(sources)),
        (A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE, &|_reader| return Check_A_Credential_Is_Not_Hardcoded_In_Source(sources)),
        (A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, &|_reader| return Check_A_Secret_Does_Not_Travel_In_A_Url(sources)),
        (CERTIFICATE_VERIFICATION_IS_NOT_DISABLED, &|_reader| return Check_Certificate_Verification_Is_Not_Disabled(sources)),
        (A_DISCARDED_ERROR_IS_EXPLAINED, &|_reader| return Check_A_Discarded_Error_Is_Explained(sources)),
        (A_SKIPPED_TEST_STATES_WHY, &|_reader| return Check_A_Skipped_Test_States_Why(sources)),
        (AN_EXCLUDED_FILE_SAYS_WHY, &|_reader| return Check_An_Excluded_File_Says_Why(sources)),
        (SUPPRESSION_DIRECTIVES_CARRY_A_REASON, &|_reader| return Check_Suppression_Directives_Carry_A_Reason(sources)),
        (WORKSPACE_MARKERS_CARRY_A_REASON, &|_reader| return Check_Workspace_Markers_Carry_A_Reason(sources)),
        (A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY, &|_reader| return Check_A_Package_Is_Named_After_Its_Directory(sources)),
        (ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, &|_reader| return Check_Atomic_Ordering_Choices_Are_Justified(sources)),
        (SEQCST_JUSTIFIED_EXPLICITLY, &|_reader| return Check_Seqcst_Justified_Explicitly(sources)),
        (RELAXED_NOT_USED_WHEN_ORDERING_MATTERS, &|_reader| return Check_Relaxed_Not_Used_When_Ordering_Matters(sources)),
        (DATA_NAMES_STAY_LOWER_SNAKE, &|reader| return Check_Data_Names_Stay_Lower_Snake(sources, reader)),
        (FILE_NAME_MATCHES_DECLARED_TYPE, &|reader| return Check_File_Name_Matches_Declared_Type(sources, reader)),
        (CONSTANTS_SPLIT_BY_EXPORT, &|reader| return Check_Go_Constants_Split_By_Export(sources, reader)),
        (GO_VARIABLES_USE_LOWER_SNAKE_CASE, &|reader| return Check_Go_Variables_Use_Lower_Snake_Case(sources, reader)),
        (EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, &|reader| return Check_Exported_Go_Functions_Use_Upper_Snake_Case(sources, reader)),
        (UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, &|reader| return Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter(sources, reader)),
        (TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE, &|reader| return Check_Go_Type_Names_Use_Camel_Case(sources, reader)),
        (PARAMETER_COUNT, &|reader| return Check_Parameter_Count(sources, reader)),
        (GO_HELPERS_PACKAGE_FIVE_INPUTS, &|reader| return Check_Go_Helpers_Package_Five_Inputs(sources, reader)),
        (DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, &|reader| return Check_Declared_Tooling_Language_For_Scripts(sources, reader)),
        (FILE_SIZE_JUSTIFICATION_TRIGGER, &|reader| return Check_File_Size_Justification_Trigger(sources, reader)),
        (ONE_THOUSAND_LINE_HARD_TRIGGER, &|reader| return Check_Go_File_Size_Hard_Trigger(sources, reader)),
        (FIVE_HUNDRED_LINE_REVIEW_TRIGGER, &|reader| return Check_Go_File_Size_Review_Trigger(sources, reader)),
        (LOWERCASE_FIRST_LETTER, &|_reader| return Check_Error_Message_Starts_Lowercase(sources)),
        (NO_TRAILING_PUNCTUATION, &|_reader| return Check_Error_Message_Has_No_Trailing_Punctuation(sources)),
        (EAGER_VS_LAZY_CONTEXT, &|_reader| return Check_Eager_Vs_Lazy_Context(sources)),
    ];

    return Findings_For_Selected_Rules(rules, reader, selected);
}

/// Runs every `rules` entry `selected` names, in table order, and collects what each
/// produces.
fn Findings_For_Selected_Rules(
    // `dyn Fn` matches the array element type `Rule_Findings` builds, above: eight distinct
    // closures need one common type, and `dyn Fn` is that type where `impl Fn` cannot be.
    rules: [(&str, &dyn Fn(&mut Reader<'_, '_>) -> Vec<Finding>); RULE_COUNT],
    reader: &mut Reader<'_, '_>,
    selected: &[RuleId],
) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for (rule, check) in rules
    {
        if Is_Rule_Selected(selected, rule)
        {
            let rule_findings = check(reader);
            findings.extend(rule_findings);
        }
    }

    return findings;
}

/// What [`Materialize_Capabilities`] already found on its own -- a failed dependency, lint
/// or policy materialization, reported rather than judged -- unconditionally, since each
/// such finding already carries its own rule and a caller that did not select it would
/// never have triggered the materialization that raises it.
fn Capability_Findings(capabilities: CapabilityMaterialization) -> Vec<Finding>
{
    let mut findings = capabilities.dependency_findings;
    findings.extend(capabilities.lint_findings);
    findings.extend(capabilities.policy_findings);

    return findings;
}

/// The whole run, once judging is done -- how many files and facts it examined, and the
/// claim its own findings support.
fn Outcome_Of(files: usize, facts: usize, findings: Vec<Finding>) -> CheckOutcome
{
    use crate::examined::{Claim_Of, Examined};

    let examined = Examined { files, facts };
    let claim = Claim_Of(&findings);

    return CheckOutcome::Judged { findings, examined, claim };
}

/// `sources`, each carrying its own resolved [`nomos_rules::SourceFile::preferred_syntax_provider`]
/// -- `OD-CAPABILITY-009`'s corrected fix, computed once here because this composition root
/// is the one place in the call chain allowed to know `nomos_lang_rust` and `nomos_lang_go`
/// by name; `nomos_rules` itself never does. Every rule this crate composes sees only the
/// enriched copy, so a subject's syntax provider identity is settled before any of them run,
/// the same "carried rather than derived" reasoning [`SourceFile::subject`] already states
/// for the field this one sits beside.
///
/// `pub(crate)` rather than private to [`Run`] alone: this crate's own `tests.rs` reaches
/// past `Run` into `crate::composition` and `crate::facts` directly, by design, to prove the
/// split-composition guarantee against the store rather than against `Run`'s one call shape
/// -- and a fixture built that way needs the identical enrichment `Run` gives every other
/// caller, not a second, differently-behaved copy of it.
pub(crate) fn Recognized_Sources(sources: &[SourceFile]) -> Vec<SourceFile>
{
    return sources
        .iter()
        .cloned()
        .map(|mut source| {
            source.preferred_syntax_provider = Recognized_Syntax_Provider(&source.path);
            return source;
        })
        .collect();
}

/// Whether `rule` is one `selected` asks for -- every rule when `selected` is empty, the same
/// "empty is everything" default `nomos_gate_orchestration::RuleSelector::include` already
/// has.
fn Is_Rule_Selected(selected: &[RuleId], rule: &str) -> bool
{
    return selected.is_empty() || selected.iter().any(|id| return id.As_Str() == rule);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::ProviderId;
    use nomos_platform_std::StdProcessLauncher;

    /// `root` is never read: `COMPLETENESS_MIRROR` alone selects none of the
    /// dependency-edges, lint-diagnostics or dependency-policy materializations, so this
    /// stays a fast, self-contained proof of `Run`'s own composing-and-judging contract
    /// rather than a second real-repository integration test -- `src/tests.rs` already
    /// carries that one, over the real syntax provider and the real rule.
    #[test]
    fn Test_Run_Should_Judge_A_Clean_Source_With_No_Findings()
    {
        let sources = vec![SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n")];
        let selected = [RuleId::New(COMPLETENESS_MIRROR)];
        let root = Path::new(".");

        let outcome = Run(&sources, RunContext { variant: Test_Variant(), root, launcher: &StdProcessLauncher }, &selected);

        let CheckOutcome::Judged { findings, examined, claim } = outcome
        else
        {
            // this fixture's own source is well-formed and the provider recognizes it; a
            // refusal here is a bug in the test's own setup, not a caller-facing failure.
            panic!("a tree the provider can read must be judged");
        };
        assert!(findings.is_empty(), "{findings:?}");
        assert_eq!(examined, crate::examined::Examined { files: 1, facts: 1 });
        assert_eq!(claim, crate::examined::Claim::Complete);
    }

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
    }

    /// `OD-CAPABILITY-009`'s corrected fix, exercised directly: a `.rs` path's enrichment
    /// must resolve to `nomos_lang_rust`'s own identity, the same identity
    /// [`crate::facts::dependency_materialization::Materialize_Syntax`]'s write side
    /// dispatches on, so the two sides agree by construction.
    #[test]
    fn Test_Recognized_Sources_Should_Populate_Preferred_Syntax_Provider()
    {
        let sources = vec![SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n")];

        let recognized = Recognized_Sources(&sources);

        assert_eq!(
            recognized.first().expect("one source in, one source out").preferred_syntax_provider,
            Some(ProviderId::New(nomos_lang_rust::PROVIDER))
        );
    }

    /// A path neither syntax provider recognizes must enrich to `None` rather than to a
    /// guess -- the same "carried rather than derived" contract
    /// [`Recognized_Syntax_Provider`] states for the field this populates.
    #[test]
    fn Test_Recognized_Sources_Should_Leave_An_Unrecognized_Path_With_No_Preferred_Provider()
    {
        let sources = vec![SourceFile::New("readme.md", nomos_model::Subject_Of_Path("readme.md"), "# hi\n")];

        let recognized = Recognized_Sources(&sources);

        assert_eq!(recognized.first().expect("one source in, one source out").preferred_syntax_provider, None);
    }
}
