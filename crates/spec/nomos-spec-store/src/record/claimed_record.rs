//! Step one: the record, read out and held for editing.

use nomos_spec_model::Record;
use nomos_spec_model::FrontMatter as RecordFrontMatter;

use crate::EditError;
use crate::RecordProjection;
use crate::StagedEdit;

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
        use crate::authoring::Why_Not_Canonical;
        use nomos_spec_model::Parse_Record;
        use nomos_spec_model::Is_Round_Trip;

        let record = Parse_Record(markdown)
            .map_err(|error| return EditError::Unreadable { cause: error.to_string() })?;

        if record.front_matter.id != self.projection.node_id
        {
            return Err(EditError::IdentityChanged {
                held: self.projection.node_id.clone(),
                staged: record.front_matter.id,
            });
        }
        if !Is_Round_Trip(markdown)
        {
            return Err(EditError::NotCanonical {
                cause: Why_Not_Canonical(markdown, &record),
            });
        }

        return Ok(record);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const CANONICAL: &str = "---\nid: D-900\ntype: decision\ntitle: A synthetic record\n\
                             status: accepted\nversion: 1\n\
                             authority: canonical-normative-record\ntags:\n  - testing\n\
                             relations:\n  - target: D-129\n    type: relates-to\n---\n\n\
                             # A synthetic record\n\n## Decision\n\nFirst paragraph.\n\n\
                             ## Rationale\n\nSecond paragraph.\n";

    #[test]
    fn Test_Markdown_Should_Return_The_Text_Held_For_Editing()
    {
        let claimed = A_Claimed_Record();

        assert_eq!(claimed.Markdown(), CANONICAL);
    }

    #[test]
    fn Test_Node_Id_Should_Name_The_Identity_Held_For_Editing()
    {
        let claimed = A_Claimed_Record();

        assert_eq!(claimed.Node_Id(), "D-900");
    }

    #[test]
    fn Test_Path_Should_Name_Where_The_Record_Currently_Lives()
    {
        let claimed = A_Claimed_Record();

        assert_eq!(claimed.Path(), "docs/records/D-900-a-synthetic-record.md");
    }

    #[test]
    fn Test_Stage_Should_Accept_A_Canonical_Edit_And_Carry_The_Claim_Forward()
    {
        let claimed = A_Claimed_Record();

        let staged =
            claimed.Stage(CANONICAL, Some("docs/records/renamed.md")).expect("stages");

        assert_eq!(staged.path, "docs/records/renamed.md");
        assert_eq!(staged.markdown, CANONICAL);
    }

    fn A_Claimed_Record() -> ClaimedRecord
    {
        return ClaimedRecord {
            projection: RecordProjection {
                node_id: "D-900".to_owned(),
                path: "docs/records/D-900-a-synthetic-record.md".to_owned(),
                revision: "v1".to_owned(),
                markdown: CANONICAL.to_owned(),
                source_hash: "sha256:same".to_owned(),
                projected_hash: "sha256:same".to_owned(),
            },
            front_matter: RecordFrontMatter {
                id: "D-900".to_owned(),
                kind: "decision".to_owned(),
                title: "A synthetic record".to_owned(),
                status: "accepted".to_owned(),
                authority: "canonical-normative-record".to_owned(),
                version: 1,
                tags: vec!["testing".to_owned()],
                relations: Vec::new(),
            },
            document_uid: 1,
        };
    }
}
