//! Materializing one compiler-backed family's fact through whichever provider the
//! composition supplies for it, and what that produced.
//!
//! One module and one function for both of `nomos-lang-rust-compiler`'s capabilities, not
//! two, for the same reason `crate::facts::policy_materialization` is one function for eight
//! repository-declared policy families: read through a port, the two have nothing left to
//! differ in. What differs is which provider answers and which rule a refusal is reported
//! under, and both are parameters.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_platform::Environment;
use nomos_rules::SourceFile;
use std::path::Path;

use crate::composed_providers::{ProjectFactProvider, SubjectFact};
use crate::facts::currency::Materialized_Or_Already_Current;

/// The root a compiler-backed provider is run over, the context its fact is filed under, the
/// store it is filed into and the environment it resolves its own working directory through
/// -- grouped into one value so [`Materialize_Compiler_Family`] names its provider as its
/// own second parameter rather than as its fifth, the same reason
/// [`super::WorkspaceReading`] groups the four a subprocess-backed provider needs.
pub struct ProjectReading<'a, Env: Environment>
{
    pub root: &'a Path,
    pub context: &'a Context,
    pub store: &'a mut MemoryFactStore,
    pub environment: &'a Env,
}

/// What materializing one compiler-backed family produced: the sources a rule can judge it
/// under, and any finding the materialization itself already raised -- the identical shape
/// [`super::PolicyMaterialization`] already has for the identical reason.
pub struct CompilerMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}

/// Runs `provider` over `reading.root`, materializes the one fact this capability's
/// `IncrementalGranularity::Project` ceiling allows for the project rooted there, and returns
/// the subject a rule can judge it under.
///
/// `rule` is the identity a refusal is reported under, because the two families this serves
/// are read by two different rules and a `ProviderUnavailable` finding carrying the wrong one
/// would be attributed to a rule that never asked. It is the only thing the two calls differ
/// in beyond the provider itself.
///
/// A failure here does not abort the run, the identical reasoning
/// [`super::Materialize_Policy`]'s own doc gives: a failed materialization returns no sources
/// and one synthetic finding naming the capability that could not be answered, rather than
/// nothing at all. That matters more here than for any other family, because this provider
/// refuses outright for a root that is not a loadable Cargo project -- an ordinary case for a
/// repository this rule is pointed at rather than a defect.
pub fn Materialize_Compiler_Family<Env: Environment>(
    reading: ProjectReading<'_, Env>,
    provider: ProjectFactProvider<Env>,
    rule: &'static str,
) -> CompilerMaterialization
{
    let context = reading.context;
    let materialized = super::Materialize_Through(
        || return provider(reading.root, context, reading.environment),
        reading.store,
        |fact, store| return Materialized_Project_Sources(fact, context, store),
        |error| return Compiler_Capability_Unavailable(rule, error),
    );

    return CompilerMaterialization { sources: materialized.sources, findings: materialized.findings };
}

/// The one project fact as the (at most one-element) source list its rule can judge -- empty
/// if the write was refused, one entry once `store` holds the fact current, whether this call
/// filed it or [`crate::facts::currency`] found the store already serving one byte-for-byte
/// identical to it. The same shape
/// [`super::policy_materialization::Materialized_Policy_Sources`] has, and for the same
/// reason: `crate::run_context::Judged` calls every rule uniformly over a source list, and a
/// whole-project capability's own list just never holds more than one.
fn Materialized_Project_Sources(fact: SubjectFact, context: &Context, store: &mut MemoryFactStore) -> Vec<SourceFile>
{
    let path = fact.path;
    let subject = fact.subject;
    if !Materialized_Or_Already_Current(fact.fact, context, store)
    {
        return Vec::new();
    }

    return vec![SourceFile::New(path, subject, String::new())];
}

/// The one finding a refused compiler-backed materialization produces -- attributed to the
/// whole tree, since the provider's failure is not about any subject this run walked, the
/// identical shape [`super::policy_materialization`]'s own refusal finding already has.
fn Compiler_Capability_Unavailable(rule: &'static str, error: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(rule),
        subject: nomos_model::Subject_Of_Path(""),
        subject_name: "workspace".to_owned(),
        applicability: Applicability::ProviderUnavailable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the {rule} capability could not be materialized, so this tree's own \
             compiler-resolved answer was not judged in this run: {error}"
        ),
        locations: Vec::new(),
    };
}
