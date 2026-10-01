//! Materializing the one `dependency.policy` fact through whichever provider the composition
//! supplies for it, and what that produced.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_platform::{Environment, ProgramLauncher};
use nomos_rules::SourceFile;

use crate::composed_providers::{SubjectFact, WorkspaceFactProvider};
use crate::facts::currency::Materialized_Or_Already_Current;

use super::WorkspaceReading;

/// What materializing the `dependency.policy` fact produced: the sources a rule can judge
/// it under, and any finding the materialization itself already raised -- the identical
/// shape [`super::LintMaterialization`] already has for the identical reason.
pub struct PolicyMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}

/// Runs `provider` over `reading.root`, materializes the one `nomos.cap.dependency.policy`
/// fact this capability's `IncrementalGranularity::WholeWorkspace` ceiling allows, and
/// returns the subject a rule can judge it under.
///
/// The port is [`WorkspaceFactProvider`] and not the [`crate::composed_providers::
/// SubjectFactsProvider`] its two siblings take, which is `OD-CAPABILITY-008`'s measured
/// distinction made structural: one fact for the whole workspace and many facts one per
/// member are two capability granularities, and a single port admitting both would be a
/// shape invented to paper over the difference rather than a shape either provider has.
///
/// A failure here does not abort the run, the identical reasoning
/// [`super::Materialize_Lint`]'s own doc gives one layer up: a failed materialization
/// returns no policy sources and one synthetic finding naming the capability that could not
/// be answered, rather than nothing at all.
pub fn Materialize_Policy<Launcher: ProgramLauncher, Env: Environment>(
    reading: WorkspaceReading<'_, Launcher, Env>,
    provider: WorkspaceFactProvider<Launcher, Env>,
) -> PolicyMaterialization
{
    let context = reading.context;
    let materialized = super::Materialize_Through(
        || return provider(reading.root, context, reading.subprocess.launcher, reading.subprocess.environment),
        reading.store,
        |fact, store| return Materialized_Policy_Sources(fact, context, store),
        Policy_Capability_Unavailable,
    );

    return PolicyMaterialization { sources: materialized.sources, findings: materialized.findings };
}

/// The one `dependency.policy` fact as the (at most one-element) source list `Check_
/// Dependency_Policy` can judge -- empty if the write was refused, one entry once `store`
/// holds the fact current, whether this call filed it or [`crate::facts::currency`] found the
/// store already serving one byte-for-byte identical to it. The same shape
/// [`super::lint_materialization::Materialized_Lint_Sources`] has for a list rather than a
/// single value: `crate::run_context::Judged` calls every rule uniformly over a source list, and a
/// whole-workspace capability's own list just never holds more than one.
///
/// A whole-workspace capability has exactly one subject, so its currency check is
/// all-or-nothing where the two per-member families beside it are per subject. That is this
/// capability's own declared `IncrementalGranularity::WholeWorkspace` ceiling showing through
/// rather than a second policy: its verdict is not attributable to one member, so there is no
/// finer subject for a check to be current about.
fn Materialized_Policy_Sources(fact: SubjectFact, context: &Context, store: &mut MemoryFactStore) -> Vec<SourceFile>
{
    let path = fact.path;
    let subject = fact.subject;
    if !Materialized_Or_Already_Current(fact.fact, context, store)
    {
        return Vec::new();
    }

    return vec![SourceFile::New(path, subject, String::new())];
}

/// The one finding a refused dependency-policy materialization produces -- the identical
/// shape [`super::lint_materialization::Lint_Capability_Unavailable`] already has for the
/// identical reason: attributed to the whole tree, since the provider's failure is not
/// about any subject this run walked.
fn Policy_Capability_Unavailable(error: &str) -> Finding
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
