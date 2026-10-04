//! [`CheckOutcomeResponse`], the check behind a run as
//! [`super::gate_run_response::GateRunResponse`] carries it.

use nomos_check_orchestration::{CheckOutcome, Claim};
use nomos_contracts::RuleId;
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
        /// Every rule this run selected whose population was empty, in the order it judged
        /// them -- `nomos_check_orchestration::Populations::Empty`, read and not recounted.
        ///
        /// Beside `complete` and never in it, which is `OD-ANALYSIS-012` version 3: such a rule
        /// judged nothing, and its zero findings would otherwise reach a caller exactly as a
        /// judgment of real subjects found clean does, while no empty population moves the
        /// claim or the disposition. Absent from the serialized response when every selected
        /// rule judged something, so a run with nothing new to say tells a caller nothing new.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        empty_populations: Vec<RuleId>,
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
            CheckOutcome::Judged { examined, claim, populations, .. } => Self::Judged {
                files: examined.files,
                facts: examined.facts,
                complete: *claim == Claim::Complete,
                empty_populations: populations.Empty().into_iter().cloned().collect(),
            },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_check_orchestration::{Examined, Populations, SupportingFactTrail};

    /// How many files a `NoFacts` outcome in these tests read before nothing was materialized.
    const READ_FILES: usize = 3;
    /// How many files a judged outcome in these tests read.
    const JUDGED_FILES: usize = 4;
    /// How many facts a judged outcome in these tests materialized.
    const JUDGED_FACTS: usize = 9;

    /// One field of a serialized value, by path.
    ///
    /// `serde_json::Value`'s own `Index` panics on a missing key, which is the failure
    /// `clippy::indexing_slicing` is denied in this workspace to prevent, so these tests read
    /// through `get` and report a missing key as `Null` rather than as a panic inside an
    /// assertion.
    fn Field_At(value: &serde_json::Value, path: &[&str]) -> serde_json::Value
    {
        const NOTHING: serde_json::Value = serde_json::Value::Null;

        let mut current = value;
        for step in path
        {
            current = current.get(step).unwrap_or(&NOTHING);
        }

        return current.clone();
    }

    #[test]
    fn Test_Every_Non_Judged_Outcome_Should_Serialize_Under_A_Name_Of_Its_Own()
    {
        assert_eq!(Field_At(&Rendered_Outcome(&CheckOutcome::Unreadable), &["outcome"]), "unreadable");
        assert_eq!(Field_At(&Rendered_Outcome(&CheckOutcome::NoSource), &["outcome"]), "no_source");
        assert_eq!(Field_At(&Rendered_Outcome(&CheckOutcome::NoFacts { files: READ_FILES }), &["outcome"]), "no_facts");
        assert_eq!(Field_At(&Rendered_Outcome(&CheckOutcome::NoFacts { files: READ_FILES }), &["files"]).as_u64(), Some(READ_FILES as u64));
    }

    /// The distinction this type exists for: two outcomes that both produce an
    /// `indeterminate` disposition and empty findings are still told apart here.
    #[test]
    fn Test_No_Source_And_No_Facts_Should_Not_Serialize_Alike()
    {
        assert_ne!(Rendered_Outcome(&CheckOutcome::NoSource), Rendered_Outcome(&CheckOutcome::NoFacts { files: 1 }));
    }

    #[test]
    fn Test_A_Judged_Outcome_Should_Carry_Its_Counts_And_Its_Claim_And_Not_Its_Findings()
    {
        let outcome = CheckOutcome::Judged { findings: Vec::new(), examined: Examined { files: JUDGED_FILES, facts: JUDGED_FACTS }, claim: Claim::Incomplete, supporting_facts: SupportingFactTrail::New(), populations: nomos_check_orchestration::Populations::New(), undeclared: nomos_check_orchestration::UndeclaredValues::New() };

        let rendered = Rendered_Outcome(&outcome);

        assert_eq!(Field_At(&rendered, &["outcome"]), "judged");
        assert_eq!(Field_At(&rendered, &["files"]).as_u64(), Some(JUDGED_FILES as u64));
        assert_eq!(Field_At(&rendered, &["facts"]).as_u64(), Some(JUDGED_FACTS as u64));
        assert_eq!(Field_At(&rendered, &["complete"]).as_bool(), Some(false));
        assert!(rendered.get("findings").is_none(), "the response's own findings field carries them, grouped: {rendered}");
    }

    /// A rule the populations below report as having judged nothing.
    const EMPTY_RULE: &str = "go-only-rule";

    /// A rule the populations below report as having judged [`JUDGED_FILES`] sources.
    const JUDGED_RULE: &str = "rust-only-rule";

    /// `OD-ANALYSIS-012` version 3, over the check a gate response carries: of two rules, the
    /// one whose population was empty is named and the one that judged real sources is not, and
    /// `complete` -- the claim -- with the counts beside it, is what it is with no population
    /// reported at all.
    #[test]
    fn Test_A_Judged_Outcome_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Claim()
    {
        let reported = Rendered_Outcome(&Complete_Over(Populations_Of(&[(EMPTY_RULE, 0), (JUDGED_RULE, JUDGED_FILES)])));
        let unreported = Rendered_Outcome(&Complete_Over(Populations::New()));

        assert_eq!(Field_At(&reported, &["empty_populations"]), serde_json::json!([EMPTY_RULE]), "{reported}");
        assert_eq!(Field_At(&reported, &["complete"]).as_bool(), Some(true), "{reported}");
        let mut without_the_list = reported.clone();
        if let Some(fields) = without_the_list.as_object_mut()
        {
            let _removed = fields.remove("empty_populations");
        }
        assert_eq!(without_the_list, unreported, "the outcome apart from the list is the outcome with no population reported");
    }

    /// A run in which every selected rule judged something answers what it answered before the
    /// list existed.
    #[test]
    fn Test_A_Judged_Outcome_With_No_Empty_Population_Should_Say_Nothing_New()
    {
        assert_eq!(Rendered_Outcome(&Complete_Over(Populations_Of(&[(JUDGED_RULE, JUDGED_FILES)]))), Rendered_Outcome(&Complete_Over(Populations::New())));
    }

    /// Each `(rule, size)` noted in order.
    fn Populations_Of(judged: &[(&str, usize)]) -> Populations
    {
        let mut populations = Populations::New();
        for (rule, size) in judged
        {
            populations.Note(RuleId::New(*rule), *size);
        }

        return populations;
    }

    /// A complete judgment that found nothing, over `populations`. The claim is fixed here, so
    /// two outcomes built by this differ in their populations and in nothing else.
    fn Complete_Over(populations: Populations) -> CheckOutcome
    {
        return CheckOutcome::Judged {
            findings: Vec::new(),
            examined: Examined { files: JUDGED_FILES, facts: JUDGED_FACTS },
            claim: Claim::Complete,
            supporting_facts: SupportingFactTrail::New(),
            populations,
            undeclared: nomos_check_orchestration::UndeclaredValues::New(),
        };
    }

    /// An outcome as a caller receives it.
    fn Rendered_Outcome(outcome: &CheckOutcome) -> serde_json::Value
    {
        return serde_json::to_value(CheckOutcomeResponse::From(outcome))
            .expect("a derived Serialize over owned data has nothing to refuse");
    }
}
