//! `function_shape`'s Go preset: the arity engine narrowed to Go sources.

use super::{Check_Function_Arity_Policy, FunctionArityPolicy, GO_PARAMETER_COUNT, Resolve_Limit, Undeclared_Limit};
use crate::checks::optional_reads;
use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

/// Reports Go functions and methods that definitely exceed the value-parameter cap — a
/// repository's own declared `nomos.cap.limits.policy` when it declares `go`'s own
/// `parameter-count-max`, the repository-wide value or the axis's declared default
/// otherwise.
#[must_use]
pub fn Check_Go_Parameter_Count(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let Some(max) = Resolve_Limit(facts, &optional_reads::GO_PARAMETER_COUNT)
    else
    {
        return vec![Undeclared_Limit(GO_PARAMETER_COUNT, optional_reads::GO_PARAMETER_COUNT.axis)];
    };
    return Check_Function_Arity_Policy(
        sources,
        facts,
        FunctionArityPolicy::New(GO_PARAMETER_COUNT, max)
            .Judging(crate::checks::populations::GO_PARAMETER_COUNT_POPULATION)
            .Allow_One_Receiver_For_Qualified_Functions(),
    );
}
