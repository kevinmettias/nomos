//! [`ReproductionResponse`], carried only by [`super::commit_response::CommitResponse`].

use nomos_spec_orchestration::Reproduction;
use nomos_spec_store::EditError;
use serde::Serialize;

/// A serializable twin of `Result<`[`nomos_spec_orchestration::Reproduction`]`, EditError>`,
/// split into three outer variants rather than nested, the same shape `crate::spec::verdict_response::
/// VerdictResponse::Compared` already uses for `Verdict::Compared`'s own `Result`.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReproductionResponse
{
    /// The store's own rendering matches what was staged, byte for byte.
    Matched
    {
        hash: String
    },
    /// The store renders something else. The commit already happened; this says the round
    /// trip did not close.
    Mismatched
    {
        hash: String
    },
    /// The store could not be asked at all -- a defect in this run rather than in the edit,
    /// since the transaction that produced this answer already committed.
    Unverifiable
    {
        cause: String
    },
}

impl ReproductionResponse
{
    pub(crate) fn From(reproduction: Result<Reproduction, EditError>) -> Self
    {
        return match reproduction
        {
            Ok(Reproduction::Matched { hash }) => Self::Matched { hash },
            Ok(Reproduction::Mismatched { hash }) => Self::Mismatched { hash },
            Err(error) => Self::Unverifiable { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Map_Every_Outcome_Of_The_Domain_Result()
    {
        assert!(matches!(
            ReproductionResponse::From(Ok(Reproduction::Matched { hash: "abc".to_owned() })),
            ReproductionResponse::Matched { hash } if hash == "abc"
        ));
        assert!(matches!(
            ReproductionResponse::From(Ok(Reproduction::Mismatched { hash: "def".to_owned() })),
            ReproductionResponse::Mismatched { hash } if hash == "def"
        ));
        assert!(matches!(
            ReproductionResponse::From(Err(EditError::Unreadable { cause: "no such record".to_owned() })),
            ReproductionResponse::Unverifiable { .. }
        ));
    }
}
