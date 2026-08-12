#![allow(clippy::missing_errors_doc)]

mod regression;
mod relocations;
mod sections;
mod judgment;
mod census;
#[cfg(test)]
mod tests;

pub use regression::Regression;
pub(crate) use relocations::Relocations;
use sections::{Body, Later, Position, Repetition};
use judgment::Judge;
use census::Census;

use crate::Template;
use crate::FillerCensus;
use crate::Hollow;
use crate::Fate;
use crate::Relocation;
use crate::DocumentFate;
use crate::MemberFate;
use crate::RegressionReport;
use crate::Archive;
use crate::Is_Filler;
use crate::IngestError;
use crate::restore::Extract;
use crate::Member;
use crate::Models_In;
use crate::Restored;
use crate::revisions::Fingerprint_Of;
use crate::PairChange;
use crate::revisions::{DOMAIN_VOLUMES, RevisionFingerprint, Walk, Within};
use nomos_spec_model::{BlockKind, RowKind, Segment, SourceBlock, Table_Rows, TableRow};
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
