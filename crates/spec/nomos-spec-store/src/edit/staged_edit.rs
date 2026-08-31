//! Step three: the edit, read and accepted, not yet inspected.

use nomos_spec_model::Record;

use crate::authoring::{Block_Changes, Identity_Changes, Relation_Changes};
use crate::ClaimedRecord;
use crate::EditError;
use crate::EditPreview;
use crate::SpecificationStore;

/// Step three: the edit, read and accepted, not yet inspected.
#[derive(Debug)]
pub struct StagedEdit
{
    pub(crate) claimed: ClaimedRecord,
    pub(crate) path: String,
    pub(crate) markdown: String,
    pub(crate) record: Record,
}

impl StagedEdit
{
    /// Step four: what committing this would change.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] if the store cannot be read.
    pub fn Preview(self, store: &SpecificationStore) -> Result<EditPreview, EditError>
    {
        use nomos_spec_model::Segment;

        let before = store.Stored_Blocks(self.claimed.document_uid)?;
        let after = Segment(&self.record.body);
        let blocks = Block_Changes(&before, &after);
        let identity = Identity_Changes(&self.claimed.front_matter, &self.record.front_matter);
        let relations = Relation_Changes(
            &self.claimed.front_matter.relations,
            &self.record.front_matter.relations,
        );
        let statements =
            store.Statement_Movements(&self.claimed.projection.node_id, &before, &after)?;

        return Ok(EditPreview {
            staged: self,
            blocks,
            identity,
            relations_added: relations.added,
            relations_removed: relations.removed,
            statements,
        });
    }
}

#[cfg(test)]
mod tests
{
    use crate::edit::fixtures::{Previewed_Edit_Of_D900, PreviewedEdit};

    #[test]
    fn Test_Preview_Should_Report_What_A_Staged_Edit_Would_Change()
    {
        let PreviewedEdit { store: _store, preview } = Previewed_Edit_Of_D900();

        assert_eq!(preview.Node_Id(), "D-900");
        assert!(!preview.Has_No_Changes());
        assert!(preview.Rename().is_none());
    }
}
