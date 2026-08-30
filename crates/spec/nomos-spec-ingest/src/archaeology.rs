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
use nomos_spec_model::{BlockKind, Row as TableRow, RowKind, Segment, SourceBlock, Table_Rows};
use std::collections::{BTreeMap, BTreeSet};

pub const SHARED_BY: u32 = 3;
