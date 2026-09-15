//! [`CheckOutcomeResponse`], the check behind a run as
//! [`super::gate_run_response::GateRunResponse`] carries it.

use nomos_check_orchestration::{CheckOutcome, Claim};
use serde::Serialize;

/// A serializable twin of [`nomos_check_orchestration::CheckOutcome`].
///
/// A twin rather than a re-export for the reason [`super`]'s own doc gives for every other
/// one in this module: the type it mirrors does not derive `Serialize`, and adding it there
/// would widen an orchestration crate's public surface on behalf of one caller's shape.
///
/// # Why a headless caller needs this at all
///
/// `nomos-cli`'s own `Render_Run` states the rule it renders by: the exit code for every
/// non-`Judged` variant is a property of *why* nothing was judged, which only the check
/// outcome carries. At a terminal those four causes each get their own message, and they
/// land on two different exit codes. Over the wire they used to arrive as nothing at all --
/// `disposition` was `indeterminate` with four empty finding buckets, which is also how a
/// judged run whose gate policy could not be read arrives. A caller could not tell a
/// repository that could not be walked from one whose analysis produced no facts from one
/// with a typo in its policy file, and those call for three different actions.
///
/// # Why `Judged` does not carry its findings
///
/// [`super::gate_run_response::GateRunResponse::findings`] already carries them, grouped by
/// what each one did to the disposition, which is strictly more than the flat list here
/// would be. Two copies of one list on one wire is the duplication `OD-GATE-011` names as a
/// defect class, so this variant carries what that grouping does *not* have: how much was
/// looked at, and whether the looking was complete.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum CheckOutcomeResponse
{
    /// The tree could not be read as a tree at all, so nothing was judged.
    Unreadable,
    /// The check layer beneath this run has its own composition self-contradictory, so no
    /// fact it produced would have been offered by anybody.
    Contradictory
    {
        /// What went wrong, as the registry's own error renders it.
        cause: String,
    },
    /// The walk found no source, so nothing was judged. A clean result here would mean only
    /// that the walk found nothing.
    NoSource,
    /// Source was read and no fact was materialized for any of it, so no claim could be
    /// resolved. A clean result here would mean only that the analysis never ran.
    NoFacts
    {
        /// How many files were read before the analysis produced nothing.
        files: usize,
    },
    /// The tree was judged. Everything it found is in the response's own `findings`.
    Judged
    {
        /// How many files this judgment read.
        files: usize,
        /// How many facts it materialized from them.
        facts: usize,
        /// Whether every rule that was asked could actually look.
        ///
        /// `nomos_check_orchestration::Claim` with its two values, written as the boolean it
        /// is rather than as a third twin type in a file of its own: an enum whose whole
        /// content is a yes-or-no is not a vocabulary a wire caller gains anything from. A
        /// `false` here is coverage debt -- some rule was unsupported or its provider could
        /// not run -- and it is what `CoveragePolicy::RequireCompleteness` acts on, so a
        /// caller reading `indeterminate` beside `"complete": false` is reading the whole of
        /// why.
        complete: bool,
    },
}

impl CheckOutcomeResponse
{
    pub(crate) fn From(outcome: &CheckOutcome) -> Self
    {
        return match outcome
        {
            CheckOutcome::Unreadable => Self::Unreadable,
            CheckOutcome::Contradictory(error) => Self::Contradictory { cause: error.to_string() },
            CheckOutcome::NoSource => Self::NoSource,
            CheckOutcome::NoFacts { files } => Self::NoFacts { files: *files },
            CheckOutcome::Judged { examined, claim, .. } => Self::Judged {
                files: examined.files,
                facts: examined.facts,
                complete: *claim == Claim::Complete,
            },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_check_orchestration::Examined;

    /// One field of a serialized value, by path.
    ///
    /// `serde_json::Value`'s own `Index` panics on a missing key, which is the failure
    /// `clippy::indexing_slicing` is denied in this workspace to prevent, so these tests read
    /// through `get` and report a missing key as `Null` rather than as a panic inside an
    /// assertion.
    fn At(value: &serde_json::Value, path: &[&str]) -> serde_json::Value
    {
        const NOTHING: serde_json::Value = serde_json::Value::Null;

        let mut current = value;
        for step in path
        {
            current = current.get(step).unwrap_or(&NOTHING);
        }

        return current.clone();
    }

    fn Rendered(outcome: &CheckOutcome) -> serde_json::Value
    {
        return serde_json::to_value(CheckOutcomeResponse::From(outcome)).expect("always serializes");
    }

    #[test]
    fn Test_Every_Non_Judged_Outcome_Should_Serialize_Under_A_Name_Of_Its_Own()
    {
        assert_eq!(At(&Rendered(&CheckOutcome::Unreadable), &["outcome"]), "unreadable");
        assert_eq!(At(&Rendered(&CheckOutcome::NoSource), &["outcome"]), "no_source");
        assert_eq!(At(&Rendered(&CheckOutcome::NoFacts { files: 3 }), &["outcome"]), "no_facts");
        assert_eq!(At(&Rendered(&CheckOutcome::NoFacts { files: 3 }), &["files"]).as_u64(), Some(3));
    }

    /// The distinction this type exists for: two outcomes that both produce an
    /// `indeterminate` disposition and empty findings are still told apart here.
    #[test]
    fn Test_No_Source_And_No_Facts_Should_Not_Serialize_Alike()
    {
        assert_ne!(Rendered(&CheckOutcome::NoSource), Rendered(&CheckOutcome::NoFacts { files: 1 }));
    }

    #[test]
    fn Test_A_Judged_Outcome_Should_Carry_Its_Counts_And_Its_Claim_And_Not_Its_Findings()
    {
        let outcome = CheckOutcome::Judged { findings: Vec::new(), examined: Examined { files: 4, facts: 9 }, claim: Claim::Incomplete };

        let rendered = Rendered(&outcome);

        assert_eq!(At(&rendered, &["outcome"]), "judged");
        assert_eq!(At(&rendered, &["files"]).as_u64(), Some(4));
        assert_eq!(At(&rendered, &["facts"]).as_u64(), Some(9));
        assert_eq!(At(&rendered, &["complete"]).as_bool(), Some(false));
        assert!(rendered.get("findings").is_none(), "the response's own findings field carries them, grouped: {rendered}");
    }
}
