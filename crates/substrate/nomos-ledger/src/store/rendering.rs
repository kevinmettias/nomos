//! The document as it goes to disk, stamped with the schema this build writes.

use crate::LedgerDocument;
use crate::LedgerError;

/// The document as it goes to disk, stamped with the schema version this build writes.
///
/// Stamped here rather than taken from the document read in: a file that keeps whatever
/// version it arrived with is a file a build without a field can rewrite while still
/// claiming to speak the newer schema.
pub(super) fn Rendered_Document(document: &LedgerDocument) -> Result<String, LedgerError>
{
    use super::SCHEMA_VERSION;

    let stamped = LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items: document.items.clone(),
    };

    let mut rendered =
        serde_json::to_string_pretty(&stamped).map_err(|error| LedgerError::Unreadable {
            cause: error.to_string(),
        })?;
    rendered.push('\n');

    return Ok(rendered);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{ItemId, ItemKind, ItemOrigin, ItemState, LedgerItem, Territory};

    fn Workable_Item(id: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: "an item".to_owned(),
            why: "because".to_owned(),
            done_when: "when it is done".to_owned(),
            kind: ItemKind::Correction,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files([format!("src/{id}.rs")]),
            state: ItemState::Ready,
            depends_on: Vec::new(),
            blocked: None,
            claim: None,
            verification: None,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            declined: None,
        };
    }

    #[test]
    fn Test_Rendered_Document_Should_Stamp_The_Current_Schema_Version_Regardless_Of_What_Was_Read()
    {
        let document = LedgerDocument {
            schema_version: 1,
            items: vec![Workable_Item("R-1")],
        };

        let rendered = Rendered_Document(&document).expect("a valid document must render");

        let parsed: LedgerDocument = serde_json::from_str(&rendered).expect("the rendered text must parse back");
        assert_eq!(parsed.schema_version, crate::SCHEMA_VERSION);
        assert_eq!(parsed.items.len(), 1);
        assert!(rendered.ends_with('\n'), "the file must end with a trailing newline");
    }
}
