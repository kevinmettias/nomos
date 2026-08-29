//! [`Handle_Spec_Sources`] and its own [`SpecSourcesResponse`].

use nomos_spec_store::StoreError;
use serde::Serialize;

use super::{AbsenceResponse, Build_Corpus_Request};

/// Reports what this store was assembled from, and what it is missing, exactly as `nomos
/// spec sources` would, and hands back a JSON-serializable response.
///
/// Builds its own `CorpusRequest` from the environment alone -- the same fields `nomos-cli`'s
/// own `main.rs::Corpus_Request` builds from `--corpus`/`--corpus-revision` or the
/// environment, minus the two argv overrides this crate has no argv to read. `CorpusRequest`
/// is configuration (which corpus, which revision), not a per-call identifier the way
/// `FindingQuery` or `ItemId` are, so this function takes no parameter for it -- the same
/// "no invented shape ahead of a real body" choice [`crate::Handle_Gate_Run`] already makes
/// for `GateCommand`'s own scope/rules selectors.
#[must_use]
pub fn Handle_Spec_Sources() -> SpecSourcesResponse
{
    use nomos_platform_std::StdFileSystem;
    use nomos_spec_orchestration::SpecCommand;

    let request = Build_Corpus_Request();

    let outcome = nomos_spec_orchestration::Run(&SpecCommand::Sources, &request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Sources(sourced) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the SpecOutcome variant
        // naming the SpecCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecSourcesResponse::From(sourced);
}

/// What a real `nomos spec sources` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecSourcesResponse
{
    /// The store assembled -- possibly with real absences, which is itself the answer this
    /// verb exists to give, not a failure.
    Assembled
    {
        /// One line per input the assembly read, in the order it was read.
        read: Vec<String>,
        /// Every input the assembly expected and did not find.
        absent: Vec<AbsenceResponse>,
    },
    /// The store itself could not be assembled at all.
    Unreadable
    {
        /// What went wrong, as `StoreError`'s own `Display` renders it.
        cause: String,
    },
}

impl SpecSourcesResponse
{
    pub(crate) fn From(result: Result<nomos_spec_orchestration::SourcesAnswer, StoreError>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Assembled {
                read: answer.read,
                absent: answer.absent.into_iter().map(AbsenceResponse::From).collect(),
            },
            Err(error) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A real end-to-end call, over whatever this session's own `NOMOS_V14_CORPUS` state
    /// happens to be. `Assemble_Corpus`'s own contract is that neither the variable nor the corpus
    /// it names being present is required -- an absent corpus reports real absences rather
    /// than refusing -- so this asserts only the outcome variant, never a specific
    /// read/absent count a local environment would make flaky.
    #[test]
    fn Test_A_Real_Call_Should_Assemble_Not_Refuse()
    {
        let response = Handle_Spec_Sources();

        assert!(matches!(response, SpecSourcesResponse::Assembled { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Assembled_Response_Should_Round_Trip_As_Json()
    {
        let response = Handle_Spec_Sources();

        let json = serde_json::to_string(&response).expect("a SpecSourcesResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecSourcesResponse always has this field");

        assert_eq!(outcome, "assembled", "{json}");
    }
}
