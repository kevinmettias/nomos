//! Which facts backed a rule's judgment, at rule grain.

use crate::examined::FactRead;

/// What a run can honestly say about the facts behind a finding, answered through the rule
/// that produced it.
///
/// Four shapes rather than one shape and an absence, because `OD-HOST-016` measured that an
/// absent trail reads as a judgment made without evidence and that this is false in three
/// distinct ways. Forty-four percent of this build's composed rules can never have a trail at
/// all; a finding raised while a capability was materialized never passed through a rule's
/// judgment; and a run whose caller kept no cache simply did not record one. Collapsing those
/// into "no supporting fact" would report the first as a gap, which is exactly the direction
/// `OD-COMPLETENESS-001` warns about.
///
/// The grain is the rule, not the finding. A per-finding answer was refused: it would need a
/// join on subject equality, which reads as exact and is false for twelve of the fifty-six
/// sites that construct a `nomos_contracts::Finding` in `nomos-rules`, because a rule's
/// finding does not always name the subject that rule read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SupportingFacts
{
    /// The rule judges source text. No fact was involved and none could have been -- derived
    /// from `nomos_rules::RuleDescriptor::subject` being `nomos_rules::SubjectKind::
    /// SourceText`, never from the rule having read nothing.
    NotFactBacked,
    /// The rule read facts, reduced to the distinct provenance tuples behind them.
    ///
    /// What this may honestly claim is exactly: the rule that produced this finding, in the
    /// call that produced it, read these capabilities from these providers at these
    /// guarantees, and these reads missed. It may not claim that any particular one of them
    /// backs any particular finding. Several tuples is the ordinary case, and all of them
    /// being misses is a real answer rather than an absence.
    Read(Vec<FactRead>),
    /// The finding was raised while a capability was materialized rather than by a rule's
    /// judgment, so no rule trail exists for it. Naming it is what stops it from being read
    /// as [`Self::Unrecorded`].
    RaisedByMaterialization,
    /// The run kept no trail for this rule -- it was not selected, or the answer was read
    /// back from a cache this run never wrote.
    Unrecorded,
}
