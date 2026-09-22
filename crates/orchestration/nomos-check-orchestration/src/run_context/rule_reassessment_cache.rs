//! Whether a composed rule's prior findings can stand in for running it again.
//!
//! [`Run`](super::Run) always starts from an empty [`RuleReassessmentCache`] and drops it at
//! the end of the call, so nothing here changes what it returns: every rule it composes
//! still runs on every call, exactly as before this file existed.
//! [`Run_Reassessing`](super::Run_Reassessing) is the identical pipeline handed a cache a
//! caller keeps across two calls instead -- the reuse `P40-INCREMENTAL-SKIP-UNCHANGED-
//! SUBJECTS` already made safe for a syntax fact, extended from "skip re-deriving a fact"
//! to "skip re-running the rule that reads it," for exactly the rules where that is honest
//! today. See [`Effective_Requires`] for why it stops at exactly those rules.
//!
//! # Which rules that covers, and what widened it
//!
//! Nothing in this file decides that. A rule is reusable when no family it reads appears in
//! the `changed` list `crate::run_context::capabilities`' own `Materialization_Tracking`
//! built, so which rules can be skipped is settled by which families can report themselves
//! *unchanged*. Until `P123-NON-SYNTAX-MATERIALIZERS-PROVE-CURRENCY` only one could --
//! `nomos.cap.syntax.items` -- and every rule declaring any other family was re-judged on
//! every call however little had moved. `P40-INCREMENTAL-DEMAND-DRIVEN-RECOMPUTE-2` was
//! declined on that measurement and named the remedy: give the other materializers the
//! currency check syntax already had. `crate::facts::currency` is that check, so this cache
//! now reaches every family without one line here changing -- which is the shape it was
//! built in.

use std::collections::BTreeMap;

use nomos_contracts::{Finding, RuleId};
use nomos_rules::{RequiredFact, RuleDescriptor, SubjectKind, DESCRIPTORS};

/// A rule's most recent real findings, and the check that tells a later call whether they
/// are still good.
#[derive(Default)]
pub struct RuleReassessmentCache
{
    findings_by_rule: BTreeMap<RuleId, Vec<Finding>>,
    recorded: u32,
}

impl RuleReassessmentCache
{
    #[must_use]
    pub fn New() -> Self
    {
        return Self { findings_by_rule: BTreeMap::new(), recorded: 0 };
    }

    /// `rule`'s findings from its last real invocation under this cache, reusable this call
    /// because nothing in `changed` names a family it reads -- `None` when it has never run
    /// under this cache, when no descriptor describes it, or when a family it reads is one
    /// `changed` names.
    ///
    /// A rule [`DESCRIPTORS`] does not describe is never reused, not skipped: the composed
    /// table and that table are proven to describe each other in both directions by
    /// `tests/contract/tests/rule_descriptors.rs`, so this is a defensive default rather
    /// than a case this crate expects to reach.
    pub(crate) fn Reusable(&self, rule: &str, changed: &[RequiredFact]) -> Option<&Vec<Finding>>
    {
        let prior = self.findings_by_rule.get(&RuleId::New(rule))?;
        let descriptor = DESCRIPTORS.iter().find(|descriptor| return descriptor.id == rule)?;

        if Invalidated_By(descriptor, changed)
        {
            return None;
        }

        return Some(prior);
    }

    /// Records what a real invocation of `rule` found, for [`Self::Reusable`] to read back
    /// on a later call.
    pub(crate) fn Record(&mut self, rule: &str, findings: Vec<Finding>)
    {
        self.findings_by_rule.insert(RuleId::New(rule), findings);
        self.recorded = self.recorded.saturating_add(1);
    }

    /// How many times [`Self::Record`] has run a real rule closure recorded, across this
    /// cache's whole lifetime -- not how many rules it holds, since the same rule recorded
    /// twice (once per call it actually ran) counts twice. Exists for a test to read a real
    /// invocation count back rather than inferring one from finding content, which a rule
    /// with genuinely unchanging output could not distinguish from a skip.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn Recorded(&self) -> u32
    {
        return self.recorded;
    }
}

/// Whether anything in `changed` makes `descriptor`'s prior findings stale.
///
/// Two independent reasons, and the second is the one that was quietly lost. A rule is stale
/// when a family it declared changed, which is [`Effective_Requires`] and is what this cache
/// was built on. It is *also* stale when the text it judges changed, and for a rule whose
/// subject is a walked source that is every `SyntaxItems` change, because a source whose
/// bytes moved always produces a new syntax fact: `crate::facts::Materialize_Syntax`'s own
/// currency check is keyed on a digest of those very bytes, so a moved source can never be
/// found already current and skipped.
///
/// [`Effective_Requires`] supplies the second reason only for a rule declaring no family at
/// all, and that was enough while every source-text rule declared nothing. It stopped being
/// enough the moment a text-judging rule declared a policy family: `requires` became
/// non-empty, the substitution stopped applying, and the rule went on being reused across
/// edits to the very text it judges. Nine rules were in that state -- among them
/// `a-credential-is-not-hardcoded-in-source`, where the damaging direction is a credential
/// added to a file and reported by a fresh run but not by a reassessing one.
///
/// Deciding it here rather than by writing `SyntaxItems` into those nine descriptors is the
/// whole point. [`RuleDescriptor::requires`] is a claim about what must be **materialized**,
/// and a rule that judges text needs no syntax fact materialized to reach its judgment;
/// writing one there to buy invalidation would make that table say something untrue about
/// every rule that did it, and would leave the next such rule to rediscover this. Staleness
/// and materialization are different questions, and this file is where the first is answered
/// -- which is the same reason [`Effective_Requires`]'s own doc gives for the substitution it
/// already makes.
///
/// Conservative by construction: it can only invalidate more than the declared families alone
/// would, never less, so nothing correctly re-run before is reused now.
fn Invalidated_By(descriptor: &RuleDescriptor, changed: &[RequiredFact]) -> bool
{
    if Effective_Requires(descriptor).iter().any(|required| return changed.contains(required))
    {
        return true;
    }

    return Judges_A_Source(descriptor) && changed.contains(&RequiredFact::SyntaxItems);
}

/// Whether the rule's subject is a walked source, so a change to that source's text is a
/// change to what the rule judges.
///
/// [`SubjectKind::Workspace`] is the one that is not: its subject is a single whole-workspace
/// fact with no per-source subject of its own, so one source's text changing is not by itself
/// a change to what it read. Asked of the declared subject rather than guessed from the
/// family list, because the family list is exactly what went stale here.
fn Judges_A_Source(descriptor: &RuleDescriptor) -> bool
{
    return matches!(descriptor.subject, SubjectKind::SourceText | SubjectKind::SourceFacts);
}

/// `descriptor.requires`, or `[RequiredFact::SyntaxItems]` when it names none.
///
/// A rule with no required family still judges its subject's own text directly -- every
/// [`nomos_rules::SubjectKind::SourceText`] rule composed in this crate does exactly that --
/// so a change to that text has to invalidate it too, and every such change already shows up
/// as a new syntax materialization: `crate::facts::Materialize_Syntax` skips a subject only
/// when the store already holds a fact keyed on that subject's own current bytes, so moved
/// bytes are always a write. Declaring the dependency here rather
/// than widening [`DESCRIPTORS`] itself keeps that table's own claim exact: a source-text
/// rule truly needs no fact *materialized* to be judged, which is a different statement from
/// what tells this cache the rule has gone stale.
fn Effective_Requires(descriptor: &RuleDescriptor) -> &'static [RequiredFact]
{
    if descriptor.requires.is_empty()
    {
        return &[RequiredFact::SyntaxItems];
    }

    return descriptor.requires;
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Every rule whose findings this cache could serve across an edit to the text it judges,
    /// before [`Invalidated_By`] answered the second question: its subject is a walked source,
    /// and the families it declares do not include the one that moves when text moves.
    ///
    /// Computed rather than listed, because a list is what goes stale. The item that produced
    /// this test named nine, which was the count of rules declaring exactly
    /// `[TestMaterialPolicy]`; the real shape is wider than one family.
    fn Rules_Declaring_No_Syntax_Family() -> Vec<&'static RuleDescriptor>
    {
        return DESCRIPTORS
            .iter()
            .filter(|descriptor| return Judges_A_Source(descriptor))
            .filter(|descriptor| return !descriptor.requires.contains(&RequiredFact::SyntaxItems))
            .filter(|descriptor| return !descriptor.requires.is_empty())
            .collect();
    }

    /// The population check the fix is really about, and the one that catches the next rule of
    /// this shape rather than the ones that exist today.
    ///
    /// Asserted over every source-judging descriptor rather than over a named few: a rule
    /// added later that declares a policy family and judges text joins this set silently, and
    /// this is what refuses to let it.
    #[test]
    fn Test_Every_Source_Judging_Rule_Should_Be_Reassessed_When_The_Syntax_Family_Changed()
    {
        let changed = [RequiredFact::SyntaxItems];

        for descriptor in DESCRIPTORS.iter().filter(|descriptor| return Judges_A_Source(descriptor))
        {
            let mut cache = RuleReassessmentCache::New();
            cache.Record(descriptor.id, Vec::new());

            assert!(
                cache.Reusable(descriptor.id, &changed).is_none(),
                "`{}` judges a source and was reused across a change to the text it judges",
                descriptor.id
            );
        }
    }

    /// The falsifier's other half: the population above is not empty, so the assertion is not
    /// vacuously true. Without this, deleting every affected rule would make the test above
    /// pass while proving nothing.
    #[test]
    fn Test_The_Affected_Population_Should_Not_Be_Empty()
    {
        let affected = Rules_Declaring_No_Syntax_Family();

        assert!(
            affected.len() >= 9,
            "the item that produced this fix measured nine such rules; found {}: {:?}",
            affected.len(),
            affected.iter().map(|descriptor| return descriptor.id).collect::<Vec<_>>()
        );
    }

    /// The clause is narrow rather than a blanket re-run. A whole-workspace rule reads one
    /// fact with no per-source subject, so one source's text changing is not by itself a
    /// change to what it read, and it stays reusable.
    #[test]
    fn Test_A_Workspace_Rule_Should_Stay_Reusable_Across_A_Source_Text_Change()
    {
        let changed = [RequiredFact::SyntaxItems];
        let workspace = DESCRIPTORS
            .iter()
            .find(|descriptor| return matches!(descriptor.subject, SubjectKind::Workspace) && !descriptor.requires.is_empty())
            .expect("this workspace composes at least one whole-workspace rule");

        let mut cache = RuleReassessmentCache::New();
        cache.Record(workspace.id, Vec::new());

        assert!(
            cache.Reusable(workspace.id, &changed).is_some(),
            "`{}` reads one whole-workspace fact and should not be re-run for a source's text",
            workspace.id
        );
    }

    /// A declared family still invalidates, which is what this cache did before and must keep
    /// doing: the new clause is additional, not a replacement.
    #[test]
    fn Test_A_Declared_Family_Should_Still_Invalidate()
    {
        let affected = Rules_Declaring_No_Syntax_Family();
        let descriptor = affected.first().expect("the population is non-empty");
        let declared = descriptor.requires.first().expect("these declare at least one family");

        let mut cache = RuleReassessmentCache::New();
        cache.Record(descriptor.id, Vec::new());

        assert!(cache.Reusable(descriptor.id, &[*declared]).is_none(), "{}", descriptor.id);
    }
}
