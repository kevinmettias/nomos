//! What one revision contains, reduced to something two revisions can be compared by.

use super::{Archive, BTreeMap, ContentHash, IngestError, RevisionFingerprint};

/// One revision's fingerprints, read without unpacking.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if an entry cannot be read as text, or if the revision
/// holds no markdown at all — an archive that fingerprints to nothing is indistinguishable
/// from one that was never opened.
pub fn Fingerprint_Revision(archive: &mut Archive, label: &str) -> Result<RevisionFingerprint, IngestError>
{
    let mut documents = BTreeMap::new();

    for entry in archive.Listing().Ending_With(".md")
    {
        let text = archive
            .Read_Text(&entry)
            .map_err(|error| return IngestError::Parse(error.to_string()))?;
        documents.insert(Within_Revision(&entry), text);
    }

    return Fingerprint_Of(label, &documents);
}

// The refusal below is an `IngestError::Parse` whose message already states its own
// reason in full, so a separate `# Errors` section would only duplicate it.
#[allow(clippy::missing_errors_doc)]
pub(crate) fn Fingerprint_Of(
    label: &str,
    documents: &BTreeMap<String, String>,
) -> Result<RevisionFingerprint, IngestError>
{
    if documents.is_empty()
    {
        return Err(IngestError::Parse(format!(
            "{label} fingerprints to no document, which reads exactly like a revision that \
             was never opened"
        )));
    }

    return Ok(RevisionFingerprint {
        label: label.to_owned(),
        documents: documents
            .iter()
            .map(|(path, text)| {
                return (path.clone(), ContentHash::Of_Normalized(text).As_String_Slice().to_owned());
            })
            .collect(),
    });
}

/// The path inside the revision, with the archive's own top directory removed.
///
/// Without this every path differs between revisions by the version in its first segment
/// and every pair reports the whole corpus twice.
pub(crate) fn Within_Revision(entry: &str) -> String
{
    return entry.split_once('/').map_or_else(|| return entry.to_owned(), |(_, rest)| return rest.to_owned());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::archive::tests::{FixturePrefix, Zip_Fixture};

    #[test]
    fn Test_Fingerprint_Revision_Should_Hash_Every_Markdown_Entry_Under_The_Label()
    {
        let mut archive = Zip_Fixture(FixturePrefix("nomos-spec-ingest-fingerprint"), "basic", &[("suite/a.md", "# A\n\nText.\n")]);

        let fingerprint = Fingerprint_Revision(&mut archive, "v14.1").expect("fingerprints");

        assert_eq!(fingerprint.label, "v14.1");
        assert_eq!(fingerprint.documents.len(), 1);
        assert!(fingerprint.documents.contains_key("a.md"));
    }

    #[test]
    fn Test_Fingerprint_Of_Should_Refuse_An_Empty_Document_Map()
    {
        let documents = BTreeMap::new();

        let refusal = Fingerprint_Of("v14.1", &documents).expect_err("must refuse");

        assert!(matches!(refusal, IngestError::Parse(_)), "{refusal}");
    }

    #[test]
    fn Test_Within_Revision_Should_Strip_The_Archives_Top_Directory()
    {
        assert_eq!(Within_Revision("nomos-spec-v15.0/01_authoring/a.md"), "01_authoring/a.md");
        assert_eq!(Within_Revision("a.md"), "a.md");
    }
}
