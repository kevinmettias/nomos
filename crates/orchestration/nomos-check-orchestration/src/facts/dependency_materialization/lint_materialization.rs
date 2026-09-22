//! Materializing `lint.diagnostics` facts through whichever provider the composition
//! supplies for them, and what that produced.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_platform::{Environment, ProgramLauncher};
use nomos_rules::SourceFile;

use crate::composed_providers::{SubjectFact, SubjectFactsProvider};
use crate::facts::currency::Materialized_Or_Already_Current;

use super::WorkspaceReading;

/// What materializing `lint.diagnostics` facts produced: the sources a rule can judge them
/// under, and any finding the materialization itself already raised -- the identical shape
/// [`super::DependencyMaterialization`] already has for the identical reason.
pub struct LintMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}

/// Runs `provider` over `reading.root`, materializes one `nomos.cap.lint.diagnostics` fact
/// per workspace member, and returns the subjects a rule can judge them under -- the
/// identical shape [`super::Materialize_Dependencies`] already has for the identical reason:
/// this is the second capability whose provider answers many facts from one launch.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero lint findings" either, the same
/// `NoFacts`-shaped vacuity [`super::Materialize_Syntax`]'s own case exists to catch one
/// layer over. So a failed materialization returns no lint sources and one synthetic
/// finding reporting which capability could not be answered, rather than nothing at all.
pub fn Materialize_Lint<Launcher: ProgramLauncher, Env: Environment>(
    reading: WorkspaceReading<'_, Launcher, Env>,
    provider: SubjectFactsProvider<Launcher, Env>,
) -> LintMaterialization
{
    let context = reading.context;
    let materialized = super::Materialize_Through(
        || return provider(reading.root, context, reading.subprocess.launcher, reading.subprocess.environment),
        reading.store,
        |facts, store| return Materialized_Lint_Sources(facts, context, store),
        Lint_Capability_Unavailable,
    );

    return LintMaterialization { sources: materialized.sources, findings: materialized.findings };
}

/// Every `lint.diagnostics` fact `store` now holds current, as the source list
/// [`nomos_rules::Check_Lint_Diagnostics`] can judge -- one per workspace member whose fact
/// this call filed, plus every member whose fact the store was already serving byte-for-byte.
///
/// Per member, the identical shape
/// [`super::dependencies::Materialized_Dependency_Sources`] has for the identical reason: a
/// member whose diagnostics did not move is not re-filed, while one whose did is.
/// [`crate::facts::currency`] is the check, and the provider's own launch it reads from has
/// already happened by the time it runs -- which is why it decides only whether the write is
/// redundant and never whether the family is produced.
fn Materialized_Lint_Sources(facts: Vec<SubjectFact>, context: &Context, store: &mut MemoryFactStore) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for member in facts
    {
        if Materialized_Or_Already_Current(member.fact, context, store)
        {
            let source = SourceFile::New(member.path, member.subject, String::new());
            sources.push(source);
        }
    }

    return sources;
}

/// The one finding a refused lint-diagnostics materialization produces -- the identical
/// shape [`super::dependencies::Dependency_Capability_Unavailable`] already has for the
/// identical reason: attributed to the whole tree, since the provider's failure is not about
/// any subject this run walked.
fn Lint_Capability_Unavailable(error: &str) -> Finding
{
    return Finding {
        address: None,
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
