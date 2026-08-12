//! Step three: the edit, read and accepted, not yet inspected.

use nomos_spec_model::Record;
use nomos_spec_model::Segment;

use crate::authoring::{Block_Changes, Identity_Changes, Relation_Changes};
use crate::claimed_record::ClaimedRecord;
use crate::edit_error::EditError;
use crate::edit_preview::EditPreview;
use crate::store::SpecificationStore;

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
