//! [`Handle_Work_Validate`] and its own [`WorkValidateResponse`].

use nomos_ledger::{LedgerDocument, Territory};
use serde::Serialize;
use std::path::Path;

/// Checks the board at `directory` against its own invariants, exactly as `nomos work
/// validate` would, and hands back a JSON-serializable response.
#[must_use]
pub fn Handle_Work_Validate(directory: &Path) -> WorkValidateResponse
{
    use super::Ledger_At;
    use nomos_platform_std::StdProcessLauncher;
    use nomos_work_orchestration::WorkCommand;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Validate,
        &mut ledger,
        &StdProcessLauncher,
        || Territory::Of_Files(std::iter::empty::<String>()),
    );

    let nomos_work_orchestration::WorkOutcome::Validate(validated) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the WorkOutcome variant
        // naming the WorkCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the WorkOutcome variant naming the WorkCommand it was given")
    };

    return WorkValidateResponse::From(validated);
}

/// What a real `nomos work validate` produced, in a shape `serde_json` can hand across a
/// wire.
///
/// The same two-state honesty shape `crate::work::work_list_response::WorkListResponse` already holds to:
/// `nomos_ledger::LedgerError` does not derive `Serialize`, so `Invalid` names both a genuine
/// invariant violation (`LedgerError::Invalid`'s own variant) and every other read failure
/// alike, by the same `Display` string `LedgerError` itself already collapses them into -- no
/// finer split is invented here than the type underneath draws.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum WorkValidateResponse
{
    /// The board satisfies its own invariants.
    Valid
    {
        /// The board, as validated.
        document: LedgerDocument,
    },
    /// The board could not be read, or it read but violated an invariant.
    Invalid
    {
        /// What went wrong, as `LedgerError`'s own `Display` renders it.
        cause: String,
    },
}

impl WorkValidateResponse
{
    pub(crate) fn From(validated: Result<LedgerDocument, nomos_ledger::LedgerError>) -> Self
    {
        return match validated
        {
            Ok(document) => Self::Valid { document },
            Err(error) => Self::Invalid { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{Scratch_Board, Scratch_Board_With_One_Item};

    #[test]
    fn Test_Validating_A_Real_Well_Formed_Board_Should_Be_Valid()
    {
        let directory = Scratch_Board();

        let response = Handle_Work_Validate(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkValidateResponse::Valid { document } = response
        else
        {
            // This fixture writes a real, empty, well-formed ledger file, so validating it
            // must succeed -- reaching `Invalid` here means the scratch fixture itself is
            // broken, not a runtime condition this test should tolerate.
            panic!("an empty, well-formed ledger satisfies every invariant Validate_Document checks");
        };
        assert!(document.items.is_empty());
    }

    #[test]
    fn Test_Validating_A_Board_With_A_Real_Violation_Should_Be_Invalid()
    {
        // `Scratch_Board_With_One_Item`'s item is `Ready` with an empty territory --
        // `nomos_ledger::store::validation::Check_Territory`'s own "workable but reserves
        // nothing" violation, not a fixture built for this test alone.
        let (directory, _id) = Scratch_Board_With_One_Item();

        let response = Handle_Work_Validate(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let WorkValidateResponse::Invalid { cause } = response
        else
        {
            // This fixture's item is `Ready` with an empty territory, a known real
            // "workable but reserves nothing" violation, so a real invalidity is the only
            // correct outcome -- reaching `Valid` here means the territory check itself
            // stopped enforcing, not a condition this test should assert around.
            panic!("an item that is workable but reserves nothing is a real invariant violation");
        };
        assert!(cause.contains("reserves nothing"), "{cause}");
    }

    #[test]
    fn Test_A_Real_Valid_Response_Should_Round_Trip_As_Json()
    {
        let directory = Scratch_Board();

        let response = Handle_Work_Validate(&directory);

        let _ignored = std::fs::remove_dir_all(&directory);

        let json = serde_json::to_string(&response).expect("a WorkValidateResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized WorkValidateResponse always has this field");

        assert_eq!(outcome, "valid", "{json}");
    }
}
