//! [`AgentExecutionOutcomeResponse`], carried only by
//! [`super::StepOutcomeResponse::ClaudeCode`].

use serde::Serialize;

/// A serializable twin of [`nomos_agent_executor_claude_code::AgentExecutionOutcome`].
///
/// `assumptions`/`unresolved_questions` are [`nomos_agent_contracts::WorkResult`]'s own two
/// fields this executor can honestly populate -- `OD-EXECUTOR-008`'s decision.
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
/// The declaration is not a field of this shape, for the reason
/// [`crate::AgentDispatchResponse`] gives and not a second reason: it is a constant of the
/// executor rather than a fact about one result, and no transport serves either shape yet,
/// so a field added now would be a shape with no caller to check it against. The two twins
/// answer the same way deliberately -- a reader who finds one should be able to predict the
/// other, and this says which way that is rather than leaving it to be inferred.
///
/// [`Test_This_Shape_Should_Project_Exactly_The_Portions_Its_Executor_Declares_Substantiated`]
/// derives that agreement from the declaration instead of restating it, so a portion added to
/// either one alone reddens there rather than rotting here.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentExecutionOutcomeResponse
{
    pub assumptions: Vec<String>,
    pub unresolved_questions: Vec<String>,
    pub denied_tool_uses: Vec<String>,
    pub is_error: bool,
    pub cost_usd: f64,
    pub duration_ms: u64,
}

impl AgentExecutionOutcomeResponse
{
    pub(crate) fn From(outcome: nomos_agent_executor_claude_code::AgentExecutionOutcome) -> Self
    {
        return Self {
            assumptions: outcome.result.assumptions,
            unresolved_questions: outcome.result.unresolved_questions,
            denied_tool_uses: outcome.denied_tool_uses,
            is_error: outcome.is_error,
            cost_usd: crate::agent::Dollars_Of(outcome.cost),
            duration_ms: outcome.duration_ms,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_agent_contracts::{Substantiation, WorkResult};
    use nomos_agent_executor_claude_code::{
        AgentExecutionOutcome, MicroDollars, WORK_RESULT_SUBSTANTIATION,
    };
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
        let wire = Serialized_Outcome_Response();
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

    /// A result that carries this executor's own declaration, converted and serialized the way
    /// a wire caller would receive it.
    ///
    /// Empty, because the portions the declaration grounds and the values they hold are two
    /// different questions and only the first is this file's: an empty `assumptions` here
    /// means the model produced none, which `String_Array` in the executor's `response.rs`
    /// guarantees by refusing an absent, non-array or non-string field.
    fn Serialized_Outcome_Response() -> Value
    {
        let outcome = AgentExecutionOutcome {
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
            cost: MicroDollars::From_Micros(1_000),
            duration_ms: 1,
        };

        let response = AgentExecutionOutcomeResponse::From(outcome);
        return serde_json::to_value(&response)
            .expect("a derived Serialize over owned strings, a bool, an f64 and a u64 has nothing to refuse");
    }
}
