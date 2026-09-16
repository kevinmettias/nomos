//! Running the selected rules over what materialization produced, and collecting what they
//! answered.
//!
//! `nomos_rules::DESCRIPTORS` is the one list that says which rules exist and what each
//! reads; this module walks it in its own order, skips the rules the selection did not name,
//! reuses one already recorded whose families all held still, and runs the rest. The two
//! mappings a rule cannot declare for itself live here rather than in that table, because
//! both are orchestration concepts: which capability slice a rule is judged over
//! ([`Judged_Sources`]), and which findings materialization raised on its own rather than
//! through a rule ([`Capability_Findings`]).

use nomos_analysis::Reader;
use nomos_contracts::Finding;
use nomos_rules::{
    SourceFile, DESCRIPTORS, RuleDescriptor,
    // The six rules of [`Judged_Sources`]' mapping. Every rule identifier this crate used to
    // name for the sake of *running* a rule is now read off `DESCRIPTORS` at run time; these
    // six are the residue `OD-RULES-027` measured and licensed.
    DEPENDENCY_COMPLETENESS, DEPENDENCY_DIRECTION, DEPENDENCY_POLICY, LINT_DIAGNOSTICS, REVIEW_FINDING, WRITE_AUTHORITY,
};

use super::{CapabilityMaterialization, Is_Rule_Selected, JudgeEnvironment, Reassessment};

/// Every finding [`Rule_Findings`] produces over `sources` and `capabilities`' own source
/// lists, plus whatever
/// [`crate::run_context::capabilities::Materialize_Capabilities`] already found on its own
/// via [`Capability_Findings`] (a failed dependency or lint materialization, reported rather
/// than judged) -- unconditionally, since each such finding already carries its own rule
/// and a caller that did not select it would never have triggered the materialization
/// that raises it.
pub(super) fn Judged_Findings(sources: &[SourceFile], capabilities: CapabilityMaterialization, env: JudgeEnvironment<'_>, reassessment: Reassessment<'_>) -> Vec<Finding>
{
    let mut reader = Reader::On(env.store, env.registry, env.context);

    let mut findings = Rule_Findings(sources, &capabilities, &mut reader, reassessment);
    findings.extend(Capability_Findings(capabilities));

    return findings;
}

/// Every finding the completeness, naming-convention, dependency-direction,
/// dependency-completeness, lint-diagnostics, dependency-policy, unread-reaches-finding,
/// cross-language-correspondence and ten text-only rules `selected` asks for produce over
/// `sources` and `capabilities`' own source lists.
///
/// The ten text-only rules (no-trailing-whitespace through no-mod-rs-files below) take only
/// `sources`, the same shape `Check_Naming_Convention` and every fact-reading rule
/// it does not: a rule implemented in `nomos-rules` but never composed here reports as
/// unenforced when it is not, so wiring one in is this crate's own territory, the same
/// "composed into nomos-check-orchestration::Run" section every one of these rules' own
/// module docs already names. Five siblings this crate also implements --
/// `no-decorative-section-dividers`, `unwrap-expect-discipline`, `panics-are-justified-
/// documented-and-validated`, `boolean-predicates` and `test-functions-use-test-subject-
/// should-behavior` -- are deliberately absent: this repository's own tree currently
/// violates all five (71, 45 and 217 findings for the first, fourth and fifth respectively,
/// checked by running each alone through a real `gate run --root .` first), and wiring a
/// rule this tree fails is a different, larger change than composing one it already
/// satisfies -- the last two are real struct fields without a predicate prefix and real
/// `#[test]` names with no `_Should_`/`_Should_Not_`, not a rule bug, so there is no version
/// of this fix that is merely a defect to correct in the rule itself. `a-disabled-test-
/// states-why`, `inline-always-requires-justification`, `no-wildcard-imports`,
/// `no-single-line-function-bodies` and `single-letter-names`, checked the same way, found
/// next to nothing to violate -- the 49, 288 and 657 findings the last three first measured
/// were, almost entirely, a rule bug each (`use super::*;`'s exemption could not see across
/// a `tests.rs` split from its `#[cfg(test)]` parent, `P27-RULES-NO-WILDCARD-IMPORTS-
/// EXEMPTION`; a bare text scan with no string-literal or comment awareness at all mistook a
/// `"fn a() {}"`-shaped test fixture for real code in 287 of 288 findings, `P27-RULES-NO-
/// SINGLE-LINE-FUNCTION-BODIES-STRING-AWARE`; a use-binding's own glob (`*`) or discard
/// (`_`) token was judged as if it were a declared name in 657 of 657 findings,
/// `P27-RULES-SINGLE-LETTER-NAMES-USE-BINDING-EXEMPTION`) -- not real violations, and the
/// one real `no-single-line-function-bodies` finding left (`tests/corpus/analysis/gamma/
/// broken.rs`, deliberately-invalid corpus content) was fixed directly rather than composed
/// around, since it cost one line. All five are composed below with their two already-wired
/// siblings.
fn Rule_Findings(sources: &[SourceFile], capabilities: &CapabilityMaterialization, reader: &mut Reader<'_, '_>, reassessment: Reassessment<'_>) -> Vec<Finding>
{
    return Findings_For_Selected_Rules(Judged { sources, capabilities }, reader, reassessment);
}

/// Runs every `DESCRIPTORS` entry `reassessment.selected` names, in table order, and collects
/// what each produces -- except one already recorded in `reassessment.cache` whose own
/// required families are all absent from `reassessment.changed`, whose prior findings are
/// reused instead of running its closure again. See
/// `crate::run_context::rule_reassessment_cache`'s own module doc for which rules that is,
/// today.
fn Findings_For_Selected_Rules(judged: Judged<'_>, reader: &mut Reader<'_, '_>, reassessment: Reassessment<'_>) -> Vec<Finding>
{
    let Reassessment { selected, cache, changed } = reassessment;

    let mut findings = Vec::new();
    for descriptor in DESCRIPTORS
    {
        if !Is_Rule_Selected(selected, descriptor.id)
        {
            continue;
        }

        if let Some(reused) = cache.Reusable(descriptor.id, changed)
        {
            findings.extend(reused.clone());
            continue;
        }

        let rule_findings = Judged_By(descriptor, &judged, reader);
        cache.Record(descriptor.id, rule_findings.clone());
        findings.extend(rule_findings);
    }

    return findings;
}

/// One descriptor's own judgment: its rule's callable, over the sources that rule reads.
///
/// Named rather than inlined because the two halves are each one call and the composition
/// is the step worth naming -- which slice a rule is judged over is this module's own
/// decision ([`Judged_Sources`] says why), and running the rule is the table's.
fn Judged_By(descriptor: &RuleDescriptor, judged: &Judged<'_>, reader: &mut Reader<'_, '_>) -> Vec<Finding>
{
    let sources = Judged_Sources(descriptor.id, judged);
    return descriptor.check.Judges(sources, reader);
}

/// The sources `rule` is judged over: a capability family's own materialized slice for the
/// six rules that read one, and the walked sources for every other rule.
///
/// # Why this mapping is a declaration and not a planner
///
/// `OD-RULES-027` measured the seventy composed entries this function replaced and found
/// that sixty-two closed over the same walked sources; these six are the whole residue.
/// Which slice a rule reads is fixed at the moment that rule is written and is not computed
/// from anything -- not from what was selected, not from what is already materialized, not
/// from what the run has done so far. That makes it the same kind of fixed, hand-written
/// mapping `OD-GATE-017` already accepted for the neighbouring axis of which facts to
/// materialize: composition, not choice.
///
/// The line it must not cross is also that record's: an entry that ever reads "this slice,
/// unless that one is already materialized" has stopped being a declared fact about a rule
/// and become the demand planner `OD-RULES-009` has declined across eight rounds. That
/// change belongs in that record, not in this function.
///
/// It lives here rather than in `nomos_rules::DESCRIPTORS` because a capability slice is an
/// orchestration concept. A descriptor table naming one would be a lower band describing an
/// upper band's shape.
fn Judged_Sources<'a>(rule: &str, judged: &Judged<'a>) -> &'a [SourceFile]
{
    return match rule
    {
        DEPENDENCY_DIRECTION | DEPENDENCY_COMPLETENESS | WRITE_AUTHORITY => &judged.capabilities.dependency.sources,
        LINT_DIAGNOSTICS => &judged.capabilities.lint.sources,
        DEPENDENCY_POLICY => &judged.capabilities.policy.sources,
        REVIEW_FINDING => &judged.capabilities.review.sources,
        _ => judged.sources,
    };
}

/// What a rule is judged over: the walked sources, and the capability slices the six rules
/// that do not read the walked ones are judged over instead.
///
/// Grouped rather than passed as two parameters so [`Findings_For_Selected_Rules`] stays
/// within this crate's own `parameter-count` limit.
struct Judged<'a>
{
    /// Every source the walk found, already enriched by
    /// `crate::run_context::Recognized_Sources`.
    sources: &'a [SourceFile],
    /// What each capability family materialized.
    capabilities: &'a CapabilityMaterialization,
}

/// What `capabilities::Materialize_Capabilities` already found on its own -- a failed
/// dependency, lint or policy materialization, reported rather than judged --
/// unconditionally, since each such finding already carries its own rule and a caller that
/// did not select it would never have triggered the materialization that raises it.
fn Capability_Findings(capabilities: CapabilityMaterialization) -> Vec<Finding>
{
    let mut findings = capabilities.dependency.findings;
    findings.extend(capabilities.lint.findings);
    findings.extend(capabilities.policy.findings);
    findings.extend(capabilities.review.findings);

    return findings;
}
