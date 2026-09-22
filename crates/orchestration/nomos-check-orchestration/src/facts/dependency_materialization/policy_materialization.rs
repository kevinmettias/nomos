//! Materializing the one `dependency.policy` fact through `cargo deny`, and what that
//! produced.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_platform::{Environment, ProgramLauncher};
use nomos_rules::SourceFile;

use std::path::Path;

use super::subprocess::Subprocess;

/// What materializing the `dependency.policy` fact produced: the sources a rule can judge
/// it under, and any finding the materialization itself already raised -- the identical
/// shape [`super::LintMaterialization`] already has for the identical reason.
pub struct PolicyMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}

/// Runs `cargo deny check bans licenses sources` over `root` through `launcher`,
/// materializes the one `nomos.cap.dependency.policy` fact this capability's
/// `IncrementalGranularity::WholeWorkspace` ceiling allows, and returns the subject a rule
/// can judge it under -- the identical shape [`super::Materialize_Lint`] already has for
/// the identical reason: this workspace's third provider with I/O of its own.
///
/// A failure here does not abort the run, the identical reasoning
/// [`super::Materialize_Lint`]'s own doc gives one layer up: a failed materialization
/// returns no policy sources and one synthetic finding reporting why, rather than nothing
/// at all.
pub fn Materialize_Policy<Launcher: ProgramLauncher, Env: Environment>(
    root: &Path,
    context: &Context,
    store: &mut MemoryFactStore,
    subprocess: Subprocess<'_, Launcher, Env>,
) -> PolicyMaterialization
{
    let materialized = super::Materialize_Through(
        || nomos_lang_rust_deny::Materialize_Workspace(root, Deny_Production(context), subprocess.launcher, subprocess.environment),
        store,
        Materialized_Policy_Sources,
        Policy_Capability_Unavailable,
    );

    return PolicyMaterialization { sources: materialized.sources, findings: materialized.findings };
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
/// otherwise, the same shape
/// [`super::lint_materialization::Materialized_Lint_Sources`] has for a list rather than a
/// single value: `crate::run_context::Judged` calls every rule uniformly over a source list, and a
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
/// -- the identical shape
/// [`super::lint_materialization::Lint_Capability_Unavailable`] already has for the
/// identical reason: attributed to the whole tree, since a `cargo deny` failure is not
/// about any subject this run walked.
fn Policy_Capability_Unavailable(error: &nomos_lang_rust_deny::DenyError) -> Finding
{
    return Finding {
        address: None,
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
