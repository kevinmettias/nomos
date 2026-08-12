//! Step one: the record, read out and held for editing.

use nomos_spec_model::Parse_Record;
use nomos_spec_model::Record;
use nomos_spec_model::RecordFrontMatter;
use nomos_spec_model::Round_Trips;

use crate::authoring::Why_Not_Canonical;
use crate::edit_error::EditError;
use crate::record_projection::RecordProjection;
use crate::staged_edit::StagedEdit;

/// Step one: the record, read out and held for editing.
///
/// Carries the projection rather than pointing at the store, so the bytes an author was
/// shown are the bytes the preview compares against.
#[derive(Debug)]
pub struct ClaimedRecord
{
    pub(crate) projection: RecordProjection,
    pub(crate) front_matter: RecordFrontMatter,
    pub(crate) document_uid: i64,
}

impl ClaimedRecord
{
    /// The markdown to edit.
    #[must_use]
    pub fn Markdown(&self) -> &str
    {
        return &self.projection.markdown;
    }

    #[must_use]
    pub fn Node_Id(&self) -> &str
    {
        return &self.projection.node_id;
    }

    #[must_use]
    pub fn Path(&self) -> &str
    {
        return &self.projection.path;
    }

    /// Step two: the edited markdown, and where it should live.
    ///
    /// A rename is `Some(path)` and nothing else. `D-129`'s first paragraph is what makes
    /// that cheap: a path is navigation, so moving one changes no identity and leaves the
    /// node, its blocks and their surrogates alone.
    ///
    /// # Errors
    ///
    /// Returns [`EditError::Unreadable`] if the text is not a record,
    /// [`EditError::IdentityChanged`] if it names a different one, and
    /// [`EditError::NotCanonical`] if this surface would not have written those bytes.
    pub fn Stage(self, markdown: &str, rename: Option<&str>) -> Result<StagedEdit, EditError>
    {
        let record = self.Accepted(markdown)?;
        let path = rename.unwrap_or(&self.projection.path).to_owned();

        return Ok(StagedEdit {
            claimed: self,
            path,
            markdown: markdown.to_owned(),
            record,
        });
    }

    /// The staged text as a record, or why this surface will not take those bytes.
    fn Accepted(&self, markdown: &str) -> Result<Record, EditError>
    {
        let record = Parse_Record(markdown)
            .map_err(|error| return EditError::Unreadable { cause: error.to_string() })?;

        if record.front_matter.id != self.projection.node_id
        {
            return Err(EditError::IdentityChanged {
                held: self.projection.node_id.clone(),
                staged: record.front_matter.id,
            });
        }
        if !Round_Trips(markdown)
        {
            return Err(EditError::NotCanonical {
                cause: Why_Not_Canonical(markdown, &record),
            });
        }

        return Ok(record);
    }
}
