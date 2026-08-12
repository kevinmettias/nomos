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
#[cfg(test)]
mod tests;

pub(crate) use extract::Extract;
pub use extract::Models_In;
pub use record::{Resolve, Restore};

use crate::Origin;
use crate::reconciliation::collision::Collision;
use crate::Restored;
use crate::Member;
use crate::IngestError;
use nomos_spec_model::{BlockKind, RowKind, Segment, SourceBlock, Table_Rows, TableRow};
use nomos_spec_store::{NodeRow, SpecificationStore, StoreError};
use core::fmt::Write as _;
use std::collections::BTreeMap;
use extract::Refuse_Collisions;

/// How many members a family line names before it falls back to the count alone.
const NAMED_IN_A_SUMMARY: usize = 3;

/// The heading whose leaves are the service descriptions.
const SERVICES: &str = "6. Systems and subsystem responsibilities";

/// The heading whose table is the canonical domain model.
const DOMAIN_MODEL: &str = "5. Canonical domain model";

const GLOSSARY: &str = "Glossary";

const EXTENDED_TERMS: &str = "Extended operational terms";

#[derive(Clone, Debug, Default)]
pub struct RestorationReport
{
    /// Every member, named. Counts are queries over this.
    pub members: Vec<Member>,
    /// Names more than one restored member claims, so none of them takes it.
    ///
    /// The corpus really does define `Capability` twice — once as a canonical domain model
    /// and once as a glossary term — and they are two nodes. Handing the bare name to
    /// whichever volume was read first would make resolution depend on directory order and
    /// answer confidently with one of two right answers.
    pub ambiguous_names: Vec<String>,
    /// Aliases something outside the restoration already owned, per alias rather than
    /// counted. Left where they are: a restoration may not repoint another authority's
    /// name.
    pub contested_aliases: Vec<String>,
}

impl RestorationReport
{
    #[must_use]
    pub fn In(&self, family: Restored) -> Vec<&Member>
    {
        return self
            .members
            .iter()
            .filter(|member| return member.family == family)
            .collect();
    }

    #[must_use]
    pub fn Named(&self, name: &str) -> Option<&Member>
    {
        return self
            .members
            .iter()
            .find(|member| return member.name == name || member.id == name);
    }

    /// Names families and their members, never a bare total.
    #[must_use]
    pub fn Summary(&self) -> String
    {
        let mut lines = Vec::new();
        for family in Restored::All()
        {
            lines.push(self.Family_Line(*family));
        }

        return lines.join("\n");
    }

    /// One family's count, and up to three of the members behind it.
    ///
    /// Naming a few is what makes a count checkable against the volume by eye; naming all
    /// of them would make the summary the report it is supposed to introduce.
    fn Family_Line(&self, family: Restored) -> String
    {
        let members = self.In(family);
        let named: Vec<&str> = members
            .iter()
            .take(NAMED_IN_A_SUMMARY)
            .map(|member| return member.name.as_str())
            .collect();
        let mut line = format!("{}: {} restored", family.Label(), members.len());
        if !named.is_empty()
        {
            let _ = write!(
                line,
                " ({}{})",
                named.join(", "),
                if members.len() > named.len() { ", …" } else { "" }
            );
        }

        return line;
    }
}
