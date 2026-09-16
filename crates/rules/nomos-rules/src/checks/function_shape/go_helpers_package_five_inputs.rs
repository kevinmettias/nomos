//! `function_shape`'s Go preset: the arity engine narrowed to Go sources.

use super::{Check_Function_Arity_Policy, FunctionArityPolicy, GO, GO_HELPERS_PACKAGE_FIVE_INPUTS, MAX_VALUE_PARAMETERS, PARAMETER_COUNT_MAX_KEY, Resolve_Limit};
use crate::{GO_LANGUAGE, SourceFile};
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

/// Reports Go functions and methods that definitely exceed the value-parameter cap — a
/// repository's own declared `nomos.cap.limits.policy` when it declares `go`'s own
/// `parameter-count-max`, the repository-wide value or the prior hardcoded default
/// otherwise.
#[must_use]
pub fn Check_Go_Helpers_Package_Five_Inputs(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let max = Resolve_Limit(facts, Some(GO), PARAMETER_COUNT_MAX_KEY, MAX_VALUE_PARAMETERS);
    return Check_Function_Arity_Policy(
        sources,
        facts,
        FunctionArityPolicy::New(GO_HELPERS_PACKAGE_FIVE_INPUTS, max)
            .For_Language(GO_LANGUAGE)
            .Allow_One_Receiver_For_Qualified_Functions(),
    );
}
