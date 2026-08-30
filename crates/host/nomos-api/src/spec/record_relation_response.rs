//! [`RecordRelationResponse`], shared by [`super::preview_response::PreviewResponse::Previewed`]
//! and [`super::committed_preview_response::CommittedPreviewResponse`].

use nomos_spec_model::RecordRelation;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_model::RecordRelation`], which derives `Deserialize`
/// but not `Serialize`.
#[derive(Debug, Serialize)]
pub struct RecordRelationResponse
{
    pub target: String,
    pub relation: String,
}

impl RecordRelationResponse
{
    pub(crate) fn From(relation: RecordRelation) -> Self
    {
        return Self { target: relation.target, relation: relation.relation };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Record_Relation()
    {
        let relation = RecordRelation { target: "D-129".to_owned(), relation: "implements".to_owned() };

        let response = RecordRelationResponse::From(relation.clone());

        assert_eq!(response.target, relation.target);
        assert_eq!(response.relation, relation.relation);
    }
}
