#![allow(clippy::missing_errors_doc)]

mod regression;
mod relocations;
mod sections;
mod judgment;
mod census;
#[cfg(test)]
mod tests;

pub use regression::Regression;
pub use relocations::Relocations;
use sections::{Later, Position, Body, Repetition};
use judgment::Judge;
use census::Census;

use crate::template::Template;
use crate::filler_census::FillerCensus;
use crate::hollow::Hollow;
use crate::fate::Fate;
use crate::relocation::Relocation;
use crate::document_fate::DocumentFate;
use crate::member_fate::MemberFate;
use crate::regression_report::RegressionReport;
use crate::archive::Archive;
use crate::overlay::Is_Filler;
use crate::phases::IngestError;
use crate::restore::Extract;
use crate::member::Member;
use crate::restore::Models_In;
use crate::restored::Restored;
use crate::revisions::Fingerprint_Of;
use crate::pair_change::PairChange;
use crate::revisions::RevisionFingerprint;
use crate::revisions::Walk;
use crate::revisions::Within;
use crate::revisions::DOMAIN_VOLUMES;
use nomos_spec_model::{BlockKind, RowKind, Segment, SourceBlock, TableRow, Table_Rows};
use std::collections::{BTreeMap, BTreeSet};

pub const SHARED_BY: u32 = 3;

pub struct Revision
{
    pub label: String,
    pub documents: BTreeMap<String, String>,
}

impl Revision
{
    pub fn Read(archive: &mut Archive, label: &str) -> Result<Self, IngestError>
    {
        let mut documents = BTreeMap::new();

        for entry in archive.Listing().Ending_With(".md")
        {
            let text = archive
                .Read_Text(&entry)
                .map_err(|error| return IngestError::Parse(error.to_string()))?;
            documents.insert(Within(&entry), text);
        }

        if documents.is_empty()
        {
            return Err(IngestError::Parse(format!(
                "{label} holds no markdown, so a regression report over it would find every \
                 family missing from a revision that was never read"
            )));
        }

        return Ok(Self {
            label: label.to_owned(),
            documents,
        });
    }

    pub fn Fingerprint(&self) -> Result<RevisionFingerprint, IngestError>
    {
        return Fingerprint_Of(&self.label, &self.documents);
    }

    #[must_use]
    pub fn Volumes(&self) -> BTreeMap<String, String>
    {
        return self
            .documents
            .iter()
            .filter(|(path, _)| return path.contains(DOMAIN_VOLUMES))
            .map(|(path, text)| {
                let name = path.rsplit_once('/').map_or(path.as_str(), |(_, name)| return name);
                return (name.to_owned(), text.clone());
            })
            .collect();
    }
}
