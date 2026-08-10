//! What one revision contains, reduced to something two revisions can be compared by.

use super::{Archive, RevisionFingerprint, IngestError, BTreeMap, ContentHash};

/// One revision's fingerprints, read without unpacking.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if an entry cannot be read as text, or if the revision
/// holds no markdown at all — an archive that fingerprints to nothing is indistinguishable
/// from one that was never opened.
pub fn Fingerprint(archive: &mut Archive, label: &str) -> Result<RevisionFingerprint, IngestError>
{
    let mut documents = BTreeMap::new();

    for entry in archive.Ending_With(".md")
    {
        let text = archive
            .Read_Text(&entry)
            .map_err(|error| return IngestError::Parse(error.to_string()))?;
        documents.insert(Within(&entry), text);
    }

    return Fingerprint_Of(label, &documents);
}

#[allow(clippy::missing_errors_doc)]
pub fn Fingerprint_Of(
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
                return (path.clone(), ContentHash::Of_Normalized(text).As_Str().to_owned());
            })
            .collect(),
    });
}

/// The path inside the revision, with the archive's own top directory removed.
///
/// Without this every path differs between revisions by the version in its first segment
/// and every pair reports the whole corpus twice.
pub(crate) fn Within(entry: &str) -> String
{
    return entry.split_once('/').map_or_else(|| return entry.to_owned(), |(_, rest)| return rest.to_owned());
}
