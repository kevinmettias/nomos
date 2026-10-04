//! [`Handle_Check_Run`] and its own [`CheckResponse`], paired in one file: the response
//! type exists only for this one handler, the same "handler beside its own response"
//! locality [`crate::response::gate_run_response`] and [`crate::correction`] both keep.
//!
//! `P62-API-CHECK-SEAM` gives this crate its first bare Check verb. `nomos-cli` exposes
//! `nomos check` as its own command, distinct from `nomos gate`, but before this file
//! existed this crate could only reach `nomos_check_orchestration::Run` transitively,
//! through `nomos_gate_orchestration::Run_Gate`, which always applies `GateCommand`'s
//! suppression, baseline and coverage policy on top of it. This calls `Run` directly, the
//! same policy-free shape `nomos-cli`'s own `nomos check` already has, walking `command.root`
//! and choosing a build variant the same way [`crate::correction`] does for correction --
//! with this crate's own [`composition::Host_Variant`] and [`sources::Walked_Sources`]
//! rather than sharing either with `nomos-cli`, since a walk and a build variant are each a
//! composition-root concern, `OD-HOST-002`'s own division.

use crate::response::UndeclaredValueResponse;
use crate::{composition, sources};
use nomos_check_orchestration::{CheckCommand, CheckOutcome, Run, RunContext};
use nomos_contracts::{Finding, RuleId};
use nomos_composer_std::{ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_rules::SourceFile;
use serde::Serialize;
use std::path::Path;

mod claim_response;
mod examined_response;

pub use claim_response::ClaimResponse;
pub use examined_response::ExaminedResponse;

/// Walks `command.root` and runs every registered rule over it -- `nomos-cli::check`'s own
/// "empty selects everything" default -- and hands back a JSON-serializable [`CheckResponse`].
#[must_use]
pub fn Handle_Check_Run(command: &CheckCommand) -> CheckResponse
{
    let Some(sources) = sources::Walked_Sources(&command.root)
    else
    {
        return CheckResponse::Unreadable;
    };

    if sources.is_empty()
    {
        return CheckResponse::NoSource;
    }

    let outcome = Judged_Sources(&sources, &command.root);

    return CheckResponse::From(outcome);
}

/// Every registered rule run over `sources`, with the build variant this crate's own
/// composition root names.
fn Judged_Sources(sources: &[SourceFile], root: &Path) -> CheckOutcome
{
    let mut store = nomos_analysis::MemoryFactStore::New();

    let providers = nomos_composer_providers::Standard_Providers();
    return Run(
        sources,
        RunContext {
            variant: composition::Host_Variant(),
            root,
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            workspace: &mut None,
            store: &mut store,
            providers: &providers,
        },
        &[],
    );
}

/// A serializable twin of [`nomos_check_orchestration::CheckOutcome`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason [`crate::response`]'s own doc gives for [`crate::response::Disposition`].
/// Kept to the same variants, in the same order, so a mismatch between the two is a compile
/// error in [`CheckResponse::From`] rather than a silent divergence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum CheckResponse
{
    Unreadable,
    Contradictory
    {
        /// What went wrong, as `RegistryError`'s own `Debug` renders it -- it does not
        /// derive `Display`, the same reason `crates/host/nomos-cli/src/check/report.rs`'s
        /// own render prints `{error:?}` too.
        cause: String,
    },
    NoSource,
    NoFacts
    {
        files: usize,
    },
    Judged
    {
        findings: Vec<Finding>,
        examined: ExaminedResponse,
        claim: ClaimResponse,
        /// Every rule this run selected whose population was empty, in the order it judged
        /// them -- `nomos_check_orchestration::Populations::Empty`, read and not recounted.
        ///
        /// Beside `claim` and never in it, which is `OD-ANALYSIS-012` version 3: such a rule
        /// judged nothing, its zero findings would otherwise read exactly like a judgment of
        /// real subjects found clean, and no empty population moves the claim. Absent from the
        /// serialized response when every selected rule judged something, so a run with
        /// nothing new to say tells a caller nothing new.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        empty_populations: Vec<RuleId>,
        /// Every value a selected rule read that the repository never declared, one entry each,
        /// with what the rule did with it -- `nomos_check_orchestration::UndeclaredValues`, read
        /// and not asked again.
        ///
        /// Beside `claim` and `empty_populations` and apart from both, which is `OD-RULES-011`
        /// version 3 decision 3: a rule that judged against a value this workspace substituted
        /// reports exactly what it would had the repository declared that value, and no value
        /// nobody declared moves the claim. Absent from the serialized response when every value
        /// a rule read was declared, so a run with nothing new to say tells a caller nothing new.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        undeclared_values: Vec<UndeclaredValueResponse>,
    },
}

impl CheckResponse
{
    pub(crate) fn From(outcome: CheckOutcome) -> Self
    {
        return match outcome
        {
            CheckOutcome::Unreadable => Self::Unreadable,
            CheckOutcome::Contradictory(cause) => Self::Contradictory { cause: format!("{cause:?}") },
            CheckOutcome::NoSource => Self::NoSource,
            CheckOutcome::NoFacts { files } => Self::NoFacts { files },
            CheckOutcome::Judged { findings, examined, claim, populations, undeclared, .. } => Self::Judged {
                findings,
                examined: ExaminedResponse::From(examined),
                claim: ClaimResponse::From(claim),
                empty_populations: populations.Empty().into_iter().cloned().collect(),
                undeclared_values: UndeclaredValueResponse::Every(&undeclared),
            },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_check_orchestration::{Claim, Examined, Populations, SupportingFactTrail, UndeclaredValues};
    use nomos_rules::{UndeclaredOutcome, UndeclaredValue, NESTING_DEPTH, PARAMETER_COUNT};
    use std::path::PathBuf;

    /// A real run over a fixture tree with a real blocking phantom claim reaches a real
    /// `Judged` outcome carrying that finding -- not `Unreadable` or `NoSource` -- proving
    /// this crate, not `nomos-cli`, can produce one.
    #[test]
    fn Test_Handle_Check_Run_Should_Judge_A_Real_Tree_And_Report_A_Real_Finding()
    {
        let root = std::env::temp_dir().join("nomos-api-check-run-judged");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a fresh directory");
        std::fs::write(
            root.join("a.rs"),
            "/// A list.\n/// Mirrored by `Test_Api_Ghost`.\npub const TABLES: &[&str] = &[];\n",
        )
        .expect("writes a fixture whose stale mirror is one real blocking finding");

        let response = Handle_Check_Run(&Command_At(root.clone()));

        let _ignored = std::fs::remove_dir_all(&root);
        match &response
        {
            CheckResponse::Judged { findings, .. } =>
            {
                assert!(findings.iter().any(|finding| return finding.summary.contains("Test_Api_Ghost")), "{findings:?}");
            }
            other => panic!("expected Judged, got {other:?}"),
        }
    }

    /// An empty tree cannot be judged, the same distinction `CheckOutcome::NoSource` already
    /// keeps apart from a clean judged run.
    #[test]
    fn Test_An_Empty_Tree_Should_Report_No_Source_Found()
    {
        let root = std::env::temp_dir().join("nomos-api-check-run-empty-tree");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates an empty directory");

        let response = Handle_Check_Run(&Command_At(root.clone()));

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(response, CheckResponse::NoSource);
    }

    /// A root that does not exist is `Unreadable`, not `NoSource`: nothing was walked at all.
    #[test]
    fn Test_A_Missing_Root_Should_Report_Unreadable()
    {
        let root = std::env::temp_dir().join("nomos-api-check-run-missing-root-does-not-exist");
        let _ignored = std::fs::remove_dir_all(&root);

        let response = Handle_Check_Run(&Command_At(root));

        assert_eq!(response, CheckResponse::Unreadable);
    }

    /// The whole point of this crate: the response a real run produces is valid JSON, and
    /// its outcome round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_From_Should_Produce_A_Response_That_Round_Trips_As_Json()
    {
        let response = CheckResponse::NoSource;

        let json = serde_json::to_string(&response)
            .expect("a derived Serialize over owned data has nothing to refuse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");

        assert_eq!(parsed.get("outcome").expect("a serialized CheckResponse always has this field"), "no_source", "{json}");
    }

    /// A rule the populations below report as having judged nothing.
    const EMPTY_RULE: &str = "go-only-rule";

    /// A rule the populations below report as having judged [`JUDGED_SOURCES`] sources.
    const JUDGED_RULE: &str = "rust-only-rule";

    /// How many sources [`JUDGED_RULE`] was judged over.
    const JUDGED_SOURCES: usize = 3;

    /// `OD-ANALYSIS-012` version 3, over the response a check run answers: of two rules, the
    /// one whose population was empty is named and the one that judged three sources is not,
    /// and every other field -- the claim among them -- is what it is with no population
    /// reported at all.
    #[test]
    fn Test_A_Judged_Response_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Claim()
    {
        let reported = Rendered_Over(Populations_Of(&[(EMPTY_RULE, 0), (JUDGED_RULE, JUDGED_SOURCES)]));
        let unreported = Rendered_Over(Populations::New());

        assert_eq!(reported.get("empty_populations"), Some(&serde_json::json!([EMPTY_RULE])), "{reported}");
        assert_eq!(reported.get("claim"), Some(&serde_json::json!("complete")), "{reported}");
        let mut without_the_list = reported.clone();
        if let Some(fields) = without_the_list.as_object_mut()
        {
            let _removed = fields.remove("empty_populations");
        }
        assert_eq!(without_the_list, unreported, "the response apart from the list is the response with no population reported");
    }

    /// A run in which every selected rule judged something answers what it answered before the
    /// list existed.
    #[test]
    fn Test_A_Judged_Response_With_No_Empty_Population_Should_Say_Nothing_New()
    {
        assert_eq!(Rendered_Over(Populations_Of(&[(JUDGED_RULE, JUDGED_SOURCES)])), Rendered_Over(Populations::New()));
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

    /// The response to a complete judgment that found nothing, over `populations`, naming no
    /// value as undeclared, as a caller receives it.
    fn Rendered_Over(populations: Populations) -> serde_json::Value
    {
        return Rendered_Beside(populations, UndeclaredValues::New());
    }

    /// The response to a complete judgment that found nothing, over `populations`, naming
    /// `undeclared`, as a caller receives it. The claim is fixed here, so two responses built by
    /// this differ in what is reported beside the claim and in nothing else.
    fn Rendered_Beside(populations: Populations, undeclared: UndeclaredValues) -> serde_json::Value
    {
        let outcome = CheckOutcome::Judged {
            findings: Vec::new(),
            examined: Examined { files: JUDGED_SOURCES, facts: JUDGED_SOURCES },
            claim: Claim::Complete,
            supporting_facts: SupportingFactTrail::New(),
            populations,
            undeclared,
        };

        return serde_json::to_value(CheckResponse::From(outcome)).expect("a derived Serialize over owned data has nothing to refuse");
    }

    /// [`NESTING_DEPTH`] read a limit no `nomos-limits.json` declared and was judged against 3, and
    /// [`PARAMETER_COUNT`] read one the repository declared, which is why it names nothing --
    /// exactly what a run records for each.
    fn One_Undeclared_And_One_Declared() -> UndeclaredValues
    {
        let mut undeclared = UndeclaredValues::New();
        undeclared.Note(
            RuleId::New(NESTING_DEPTH),
            vec![UndeclaredValue {
                family: "limits",
                declared_in: "nomos-limits.json",
                key: "nesting-depth-max",
                language: None,
                outcome: UndeclaredOutcome::JudgedAgainst { value: "3".to_owned() },
            }],
        );
        undeclared.Note(RuleId::New(PARAMETER_COUNT), Vec::new());

        return undeclared;
    }

    /// `OD-RULES-011` version 3 decision 3, over the response a check run answers: of two rules,
    /// the one that read a value nobody declared is named with it and the one that read a declared
    /// value is not, beside the population and apart from it, and every other field -- the claim
    /// among them -- is what it is with no value named.
    #[test]
    fn Test_A_Judged_Response_Should_Name_Only_The_Undeclared_Value_Beside_An_Unchanged_Claim()
    {
        let reported = Rendered_Beside(Populations_Of(&[(EMPTY_RULE, 0)]), One_Undeclared_And_One_Declared());
        let unreported = Rendered_Beside(Populations_Of(&[(EMPTY_RULE, 0)]), UndeclaredValues::New());

        let named = serde_json::json!([{
            "rule": NESTING_DEPTH,
            "family": "limits",
            "declared_in": "nomos-limits.json",
            "key": "nesting-depth-max",
            "language": null,
            "outcome": "judged_against",
            "value": "3",
        }]);
        assert_eq!(reported.get("undeclared_values"), Some(&named), "{reported}");
        assert!(!reported.to_string().contains(PARAMETER_COUNT), "{reported}");
        assert_eq!(reported.get("empty_populations"), Some(&serde_json::json!([EMPTY_RULE])), "{reported}");
        assert_eq!(reported.get("claim"), Some(&serde_json::json!("complete")), "{reported}");
        let mut without_the_list = reported.clone();
        if let Some(fields) = without_the_list.as_object_mut()
        {
            let _removed = fields.remove("undeclared_values");
        }
        assert_eq!(without_the_list, unreported, "the response apart from the list is the response with no value named");
    }

    /// A run in which every value its rules read was declared answers what it answered before the
    /// list existed.
    #[test]
    fn Test_A_Judged_Response_With_Every_Value_Declared_Should_Say_Nothing_New()
    {
        let mut declared = UndeclaredValues::New();
        declared.Note(RuleId::New(PARAMETER_COUNT), Vec::new());

        assert_eq!(Rendered_Beside(Populations::New(), declared), Rendered_Over(Populations::New()));
    }

    /// A bare `CheckCommand` over `root` -- this verb selects everything by default, so the
    /// root is the whole of what a caller supplies.
    fn Command_At(root: PathBuf) -> CheckCommand
    {
        return CheckCommand { root };
    }
}
