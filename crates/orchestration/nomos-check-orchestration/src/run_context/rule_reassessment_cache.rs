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

use std::collections::BTreeMap;

use nomos_contracts::{Finding, RuleId};
use nomos_rules::{RequiredFact, RuleDescriptor, DESCRIPTORS};

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

        if Effective_Requires(descriptor).iter().any(|required| return changed.contains(required))
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

/// `descriptor.requires`, or `[RequiredFact::SyntaxItems]` when it names none.
///
/// A rule with no required family still judges its subject's own text directly -- every
/// [`nomos_rules::SubjectKind::SourceText`] rule composed in this crate does exactly that --
/// so a change to that text has to invalidate it too, and every such change already shows up
/// as a new syntax materialization: `crate::facts::Materialize_Syntax` writes that family
/// unconditionally on every call, for every source. Declaring the dependency here rather
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
