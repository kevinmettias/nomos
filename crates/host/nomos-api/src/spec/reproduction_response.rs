//! [`ReproductionResponse`], carried only by [`super::spec_commit_response::SpecCommitResponse`].

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
