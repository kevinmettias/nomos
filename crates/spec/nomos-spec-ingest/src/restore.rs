//! I5 — the families v15.0 destroyed, restored from v14.36 as resolvable nodes.
//!
//! Every member is minted from the document rather than from a list kept here. A hand-kept
//! inventory of 170 members is a second authority that drifts from the corpus the moment
//! either changes, and the whole point of the restoration is that the corpus is the source.
//! So a family declares which volume it lives in and how its members are recognised, and
//! the members are whatever that recognition finds.
//!
//! Identifiers are minted from what the document already says — its own numbering for the
//! appendices and the roadmap, the authored name for a service, a glossary term or a
//! canonical domain model. A name the corpus did not give would be an identity this build
//! invented, and re-running the restoration against a corrected corpus would silently mint
//! a second one. Two members minting the same identifier is refused rather than merged.
//!
//! Counts are not asserted here. They live in `tests/corpus/families/counts.json` with the
//! extraction that produced each one, per D-132.

mod extract;
mod record;
mod restoration_report;
#[cfg(test)]
mod tests;

pub(crate) use extract::Extract_Members;
pub use extract::Models_In;
pub use record::{Resolve_Model_Uid, Restore_Members};
pub use restoration_report::RestorationReport;

use crate::Origin;
use crate::reconciliation::collision::Collision;
use crate::Restored;
use crate::Member;
use crate::IngestError;
use nomos_spec_model::{BlockKind, RowKind, Segment, SourceBlock, Table_Rows, TableRow};
use nomos_spec_store::{NodeRow, SpecificationStore, StoreError};
use std::collections::BTreeMap;
use extract::Refuse_Collisions;

/// The heading whose leaves are the service descriptions.
const SYSTEMS_HEADING: &str = "6. Systems and subsystem responsibilities";

/// The heading whose table is the canonical domain model.
const DOMAIN_MODEL: &str = "5. Canonical domain model";

const GLOSSARY: &str = "Glossary";

const EXTENDED_TERMS: &str = "Extended operational terms";
