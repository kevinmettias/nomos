use super::{Archive, DOMAIN_VOLUMES, IngestError, RevisionFingerprint, Within_Revision};
use std::collections::BTreeMap;

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
            documents.insert(Within_Revision(&entry), text);
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
        use super::Fingerprint_Of;

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
