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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::archive::tests::Zip_Fixture;

    #[test]
    fn Test_Read_Should_Collect_Every_Markdown_Entry_Under_Its_Revision_Local_Path()
    {
        let mut archive = Zip_Fixture(
            "nomos-spec-ingest-archaeology-revision",
            "read-collects-markdown",
            &[("v15.0/a.md", "# A\n"), ("v15.0/nested/b.md", "# B\n"), ("v15.0/skip.txt", "not markdown")],
        );

        let revision = Revision::Read(&mut archive, "v15.0").expect("reads");

        assert_eq!(revision.label, "v15.0");
        assert_eq!(revision.documents.len(), 2);
        assert_eq!(revision.documents.get("a.md"), Some(&"# A\n".to_owned()));
        assert_eq!(revision.documents.get("nested/b.md"), Some(&"# B\n".to_owned()));
    }

    #[test]
    fn Test_Read_Should_Refuse_An_Archive_With_No_Markdown()
    {
        let mut archive =
            Zip_Fixture("nomos-spec-ingest-archaeology-revision", "read-refuses-empty", &[("v15.0/notes.txt", "text")]);

        let refusal = Revision::Read(&mut archive, "v15.0").err().expect("must refuse");

        assert!(matches!(refusal, IngestError::Parse(_)));
        assert!(format!("{refusal}").contains("holds no markdown"));
    }

    #[test]
    fn Test_Fingerprint_Should_Reduce_Documents_To_A_Content_Hash_Per_Path()
    {
        let revision = Revision {
            label: "v15.0".to_owned(),
            documents: BTreeMap::from([("a.md".to_owned(), "# A\n".to_owned())]),
        };

        let fingerprint = revision.Fingerprint().expect("fingerprints");

        assert_eq!(fingerprint.label, "v15.0");
        assert_eq!(fingerprint.documents.len(), 1);
        assert!(fingerprint.documents.contains_key("a.md"));
    }

    #[test]
    fn Test_Volumes_Should_Keep_Domain_Volume_Documents_Reduced_To_A_Bare_Filename()
    {
        let revision = Revision {
            label: "v15.0".to_owned(),
            documents: BTreeMap::from([
                (format!("{DOMAIN_VOLUMES}02-core-architecture.md"), "# Core\n".to_owned()),
                ("00-index.md".to_owned(), "# Index\n".to_owned()),
            ]),
        };

        let volumes = revision.Volumes();

        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes.get("02-core-architecture.md"), Some(&"# Core\n".to_owned()));
    }
}
