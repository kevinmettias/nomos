// clippy::missing_errors_doc, for this module and the submodules below it. It fires on
// `Regression_Between_Revisions`, `Revision::Read` and `Revision::Fingerprint`, and all three fail the one
// way: an `IngestError::Parse` whose message already states its own reason in full — "holds
// no markdown, so a regression report over it would find every family missing from a
// revision that was never read", "has no domain volumes, so there is no family to ask
// after". An `# Errors` section would be a second copy of that sentence and nothing checks
// a copy against the message it duplicates. Module-wide rather than per function because
// the rule is the same for anything added here: a refusal that needs a separate `# Errors`
// paragraph is a refusal whose message does not say why, and that is a defect to fix in the
// message rather than to document beside it.
#![allow(clippy::missing_errors_doc)]

mod regression;
mod relocations;
mod revision;
mod sections;
mod judgment;
mod census;
#[cfg(test)]
mod tests;

pub use regression::Regression_Between_Revisions;
pub(crate) use relocations::Relocations_Between;
pub use revision::Revision;
use sections::{Body, Later, Position, Repetition};
use judgment::Judge_Member;
use census::Census_Fillers;

use crate::Template;
use crate::FillerCensus;
use crate::Hollow;
use crate::Fate;
use crate::Relocation;
use crate::DocumentFate;
use crate::MemberFate;
use crate::RegressionReport;
use crate::Archive;
use crate::Get_Filler_Pattern;
use crate::IngestError;
use crate::restore::Extract_Members;
use crate::Member;
use crate::Models_In;
use crate::Restored;
use crate::revisions::Fingerprint_Of;
use crate::PairChange;
use crate::revisions::{DOMAIN_VOLUMES, RevisionFingerprint, Walk_Revisions, Within_Revision};
use nomos_spec_model::{BlockKind, Normalize_Whitespace, Row as TableRow, RowKind, Segment, SourceBlock, Table_Rows};
use std::collections::{BTreeMap, BTreeSet};

pub const SHARED_BY: u32 = 3;

/// The shortest normalized body eligible to be judged a template at all, in characters.
///
/// Repetition is necessary and not sufficient evidence that a body is filler, which
/// `OD-SPEC-004` version 3 decided and says why. This number is derived from the gap between
/// three measured populations rather than chosen to bound a count. Over the sibling suites
/// the longest body that collided by accident is 30 characters. Over the domain volumes the
/// shortest body repeated by design is 43. Every entry in `FILLER_PATTERNS` is at least 50,
/// and `Get_Filler_Pattern` is substring containment, so nothing this repository has ever
/// declared to be filler is shorter than that. Between 31 and 42 the distribution is empty
/// in both corpora.
///
/// Both boundaries are load-bearing. Below 31 the accidental collisions come back: 102 of
/// the 103 sibling violations were connectives and labels, one of them three characters of
/// horizontal rule carried by 562 sections. Above 43 the suite edition line is dropped for
/// being short, which is the right answer for the wrong reason -- that line is contentful
/// text a corpus ought to have to declare, and a floor that swallows it stops being right
/// the first time a suite repeats a short line it meant to. So a floor at 50, which would
/// look natural because it matches the declared vocabulary, is refused.
///
/// Within the band the two errors are not symmetric. A floor set too low reports a fragment
/// somebody files an item about; one set too high hides a hollowing, which is the failure
/// `OD-SPEC-004` was written about in the first place -- the absence of a finding being
/// indistinguishable from the absence of the problem. So the value is taken from the lower
/// half of the band rather than its middle.
///
/// Derived rather than stipulated, so it is re-derivable: when the corpora move, measure the
/// longest accidental collision and the shortest deliberate repetition again and check that
/// this still lies between them.
pub const TEMPLATE_FLOOR: usize = 36;

/// Whether a body is long enough to be judged a template at all.
///
/// Eligibility is decided before repetition is counted, and whether a corpus declared the
/// text its own layout repeats is decided after. Two mechanisms rather than one composite
/// heuristic, so neither has to know about the other and either can be built first.
///
/// Counted over the normalized text, in characters, because that is what `normalized_hash`
/// groups by and what the measurement behind [`TEMPLATE_FLOOR`] counted. A body that is one
/// downward arrow is one character here and not the three bytes it occupies -- and that
/// arrow, repeated nine times, is one of the violations this floor exists to stop.
#[must_use]
pub fn Is_Template_Eligible(text: &str) -> bool
{
    return Normalize_Whitespace(text).chars().count() >= TEMPLATE_FLOOR;
}
