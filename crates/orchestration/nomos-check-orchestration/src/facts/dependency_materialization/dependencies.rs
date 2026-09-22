//! `dependency.edges` facts, materialized by whichever provider the composition supplies for
//! them -- `cargo metadata`, in every composition this repository ships.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_platform::{Environment, ProgramLauncher};
use nomos_rules::SourceFile;

use crate::composed_providers::{SubjectFact, SubjectFactsProvider};
use crate::facts::currency::Materialized_Or_Already_Current;

use super::DependencyMaterialization;
use super::WorkspaceReading;

/// Runs `provider` over `reading.root`, materializes one `nomos.cap.dependency.edges` fact
/// per workspace member, and returns the subjects a rule can judge them under.
///
/// A second, independent materialization step beside [`super::Materialize_Syntax`] rather
/// than a generalization of it: `dependency.edges` has exactly one consumer today and
/// nothing to share or schedule against `syntax.items`'s own materialization, so composing
/// this as one more declared step is `OD-HOST-004`'s "composition, not choice" again, not
/// a case for the shared demand planner `ARC-ROADMAP-001` still leaves for later.
///
/// A failure here does not abort the run -- the syntax rules still judge what they always
/// did -- but it must not silently read as "zero dependency findings" either, which is
/// exactly the vacuity [`super::Materialize_Syntax`]'s own `NoFacts` case exists to catch
/// one layer over. So a failed materialization returns no dependency sources and one
/// synthetic finding reporting why, rather than nothing at all. That finding names the
/// capability that could not be answered, which is why the provider's refusal reaches this
/// function as a value rather than being swallowed inside it.
pub fn Materialize_Dependencies<Launcher: ProgramLauncher, Env: Environment>(
    reading: WorkspaceReading<'_, Launcher, Env>,
    provider: SubjectFactsProvider<Launcher, Env>,
) -> DependencyMaterialization
{
    let context = reading.context;
    let materialized = super::Materialize_Through(
        || return provider(reading.root, context, reading.subprocess.launcher, reading.subprocess.environment),
        reading.store,
        |facts, store| return Materialized_Dependency_Sources(facts, context, store),
        Dependency_Capability_Unavailable,
    );

    return DependencyMaterialization { sources: materialized.sources, findings: materialized.findings };
}

/// Every `dependency.edges` fact `store` now holds current, as the source list
/// [`nomos_rules::Check_Dependency_Direction`] can judge -- one per workspace member whose
/// fact this call filed, plus every member whose fact the store was already serving
/// byte-for-byte.
///
/// Per member, not per workspace: the composed provider answers one fact per member, so a
/// manifest edit under one member leaves every other member's fact untouched and only the
/// edited one is re-filed. [`crate::facts::currency`] is the check, and its own doc is why a
/// skipped write here is not the demand planner `OD-RULES-009` declines -- the provider's own
/// read has already happened.
fn Materialized_Dependency_Sources(facts: Vec<SubjectFact>, context: &Context, store: &mut MemoryFactStore) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    for package in facts
    {
        if Materialized_Or_Already_Current(package.fact, context, store)
        {
            let source = SourceFile::New(package.path, package.subject, String::new());
            sources.push(source);
        }
    }

    return sources;
}

/// The one finding a refused dependency-edges materialization produces.
///
/// Attributed to the whole tree (`Subject_Of_Path("")`, the root's own subject per
/// `nomos_model::path`'s convention) rather than to any one file, because the provider's
/// failure is not about any subject this run walked -- it is about whether the dependency
/// capability could answer at all. [`Applicability::ProviderUnavailable`] because the
/// provider is registered and offered; it ran and did not answer, which is exactly that
/// variant's own distinction from `MissingCapability`.
fn Dependency_Capability_Unavailable(error: &str) -> Finding
{
    return Finding {
        address: None,
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
