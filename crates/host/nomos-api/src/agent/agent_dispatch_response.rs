//! [`AgentDispatchResponse`], what [`super::Handle_Agent_Execute`] and
//! [`super::Handle_Agent_Judge_Role`] answer.

use serde::Serialize;

/// A serializable twin of [`nomos_agent_orchestration::AgentDispatchOutcome`].
///
/// `Executed`'s `assumptions`/`unresolved_questions` are
/// [`nomos_agent_contracts::WorkResult`]'s own two fields this executor can honestly
/// populate -- `OD-EXECUTOR-008`'s decision.
///
/// The other four -- `plan`, `claims`, `tests` and `requested_verification` -- are not
/// projected, and this shape says why rather than leaving it to a convention:
/// [`nomos_agent_executor_claude_code::WORK_RESULT_SUBSTANTIATION`], the declaration the
/// producing executor publishes beside its schema, declares exactly those four
/// `Unsubstantiated` and exactly the two carried here `Substantiated`. `OD-EXECUTOR-011`'s
/// criterion for a projection that omits a portion is that it be able to say why, and that
/// declaration is the why -- a portion declared unsubstantiated says nothing about the task
/// whatever it holds, so omitting it drops nothing a caller could act on. The field list
/// above is therefore the declaration itself rather than a summary of it: the portions a
/// caller may act on are exactly the portions a caller receives.
///
/// The declaration is not a field of this shape. It is a constant of the executor rather
/// than a fact about one result, so carrying it would repeat one sentence on every response;
/// and nothing here would read it -- no transport serves this shape yet, so a field added
/// now would be a shape with no caller to check it against, which is the premature-surface
/// caution this crate's own module doc names. The `Answered` and `Unavailable` variants
/// project no work result at all, so the criterion above does not reach them.
///
/// **The variants are named for the package kind that answered, and each carries the family
/// that answered it.** They used to be `ClaudeCode` and `Ollama`, which was the same
/// plugin-boundary leak `OD-ROADMAP-005` decision 2 closed one layer down: a wire shape whose
/// tag was a vendor. `family` keeps what that tag told a caller -- which backend answered --
/// and takes it from the declaration that resolved rather than from a match arm, which is the
/// only place it is knowable once a profile can resolve a backend nobody typed.
///
/// [`Test_This_Shape_Should_Project_Exactly_The_Portions_Its_Executor_Declares_Substantiated`]
/// derives that agreement from the declaration instead of restating it, so a portion added to
/// either one alone reddens there rather than rotting here.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum AgentDispatchResponse
{
    /// An `AgentExecutorPackage` answered.
    Executed
    {
        family: String, assumptions: Vec<String>, unresolved_questions: Vec<String>,
        denied_tool_uses: Vec<String>, is_error: bool, cost_usd: f64, duration_ms: u64
    },
    /// A `ModelBackendPackage` answered.
    ///
    /// No `denied_tool_uses`, no `is_error`, no `cost_usd` and no `duration_ms`, because the
    /// port this projects establishes none of them and a zero here would be a number nothing
    /// measured.
    Answered
    {
        family: String, response: String
    },
    /// The target that was reached did not answer.
    Unavailable
    {
        family: String, reason: String
    },
    /// No backend was selected, so nothing was dispatched and there is no backend this
    /// response is about.
    ///
    /// Distinct from [`Self::Unavailable`], which reports a backend that was chosen and then
    /// did not answer. A caller that cannot tell the two apart cannot act on either: one is a
    /// backend to fix, the other is a declaration to change.
    NotSelected
    {
        absence: String
    },
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_agent_contracts::{AgentExecution, MicroDollars, Substantiation, WorkResult};
    use nomos_agent_executor_claude_code::WORK_RESULT_SUBSTANTIATION;
    use nomos_agent_orchestration::AgentDispatchOutcome;
    use serde_json::Value;

    /// The claim the doc comment above makes, derived rather than restated: every portion of a
    /// `WorkResult` is carried here exactly when the executor's own declaration says it is
    /// substantiated.
    ///
    /// Both directions are asserted by the one comparison, and that is what makes the field
    /// list above the declaration rather than a copy of it -- a portion that moves in either
    /// place without the other is red, whichever way it moved.
    #[test]
    fn Test_This_Shape_Should_Project_Exactly_The_Portions_Its_Executor_Declares_Substantiated()
    {
        let wire = Serialized_Execution_Response();
        // Every field by name, deliberately without a `..`: a seventh portion added to the
        // declaration is a compile error here rather than a portion nothing checks.
        let Substantiation {
            plan,
            claims,
            tests,
            requested_verification,
            assumptions,
            unresolved_questions,
        } = &WORK_RESULT_SUBSTANTIATION;

        for (field, portion) in [
            ("plan", plan),
            ("claims", claims),
            ("tests", tests),
            ("requested_verification", requested_verification),
            ("assumptions", assumptions),
            ("unresolved_questions", unresolved_questions),
        ]
        {
            assert_eq!(
                wire.get(field).is_some(),
                portion.Is_Substantiated(),
                "this shape carries {field} exactly when its executor declares it substantiated"
            );
        }
    }

    /// The `Executed` variant of a result that carries this executor's own declaration,
    /// serialized the way a wire caller would receive it.
    ///
    /// Empty, because the portions the declaration grounds and the values they hold are two
    /// different questions and only the first is this file's: an empty `assumptions` here
    /// means the model produced none, which `String_Array` in the executor's `response.rs`
    /// guarantees by refusing an absent, non-array or non-string field.
    fn Serialized_Execution_Response() -> Value
    {
        let execution = AgentExecution {
            result: WorkResult {
                plan: None,
                claims: Vec::new(),
                tests: Vec::new(),
                requested_verification: None,
                assumptions: Vec::new(),
                unresolved_questions: Vec::new(),
                substantiation: WORK_RESULT_SUBSTANTIATION,
            },
            denied_tool_uses: Vec::new(),
            is_error: false,
            spend: MicroDollars::From_Micros(1_000),
            duration_ms: 1,
        };

        let response = AgentDispatchResponse::From(AgentDispatchOutcome::Executed {
            family: nomos_agent_executor_claude_code::FAMILY.to_owned(),
            execution,
        });
        return serde_json::to_value(&response)
            .expect("a derived Serialize over owned strings, a bool, an f64 and a u64 has nothing to refuse");
    }
}
