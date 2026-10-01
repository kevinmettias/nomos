//! Whether a capability slice's population moved since the reassessment cache last saw it -- the
//! half of "changed" a store's write counter cannot see.
//!
//! `crate::run_context::capabilities`' `Materialization_Tracking` names a family changed when the
//! store gained a write, which is the right signal for a fact that moved and the wrong one for a
//! slice that shrank: a workspace member removed, a declared build dropped, a provider that answered
//! last call refusing this one. None of those writes anything, so the family read as unchanged and
//! `Run_Reassessing` reused the rule's findings about members and builds that were no longer there.
//! What a rule judged over a slice sees is the slice's sources and the refusals raised beside them,
//! so that is the population compared here, and a population that moved is a change.
//!
//! This decides nothing about what is produced: every section a selection demands has already run
//! in full before it is read, and it reads no store state. It compares what this call produced
//! with what the last one did -- the side of the line `crate::facts::currency` is on, one step
//! later.

use nomos_rules::RequiredFact;

use super::{CapabilityMaterialization, MaterializedCapability, RuleReassessmentCache};

/// Adds to `changed` every slice family whose population differs from the one `cache` recorded
/// for it the call before, and records this call's.
pub(super) fn Note_Moved_Populations(capabilities: &CapabilityMaterialization, cache: &mut RuleReassessmentCache, changed: &mut Vec<RequiredFact>)
{
    let slices = [
        (RequiredFact::DependencyEdges, &capabilities.dependency),
        (RequiredFact::LintDiagnostics, &capabilities.lint),
        (RequiredFact::DependencyPolicy, &capabilities.policy),
        (RequiredFact::ReviewFindings, &capabilities.review),
        (RequiredFact::CopyClones, &capabilities.copy_clones),
        (RequiredFact::NestedLocks, &capabilities.nested_locks),
        (RequiredFact::CsharpConditional, &capabilities.csharp_conditional),
        (RequiredFact::GoDiscardedValues, &capabilities.go_discarded_values),
    ];
    for (family, slice) in slices
    {
        if cache.Population_Moved(family, Population_Of(slice)) && !changed.contains(&family)
        {
            changed.push(family);
        }
    }
}

/// One slice's population, in an order that does not depend on the order it was produced in:
/// each source's path, subject and answering provider, and each refusal's rule, subject,
/// applicability and words.
fn Population_Of(slice: &MaterializedCapability) -> Vec<String>
{
    let mut population: Vec<String> = slice
        .sources
        .iter()
        .map(|source| return format!("source\t{}\t{}\t{:?}", source.path, source.subject, source.answered_by))
        .collect();
    population.extend(
        slice
            .findings
            .iter()
            .map(|finding| return format!("refusal\t{}\t{}\t{}\t{}", finding.rule, finding.subject_name, finding.applicability.Label(), finding.summary)),
    );
    population.sort();

    return population;
}
