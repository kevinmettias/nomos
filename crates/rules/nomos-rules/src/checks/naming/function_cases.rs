//! The case a function's name is judged against, on each side of visibility.
//!
//! `OD-RULES-035` decision 7 has a Rust function read its case through the refinement of
//! `function` its visibility selects, so one source can hold names judged against two cases.
//! Which side a function is on is the syntax fact's own word for it: exactly `pub` is exported,
//! and everything else -- private, `pub(crate)`, `pub(super)`, `pub(in path)`, or a trait
//! declaration's member, which declares no visibility of its own -- is not. That is
//! code-standards' Rust test too (`is_Symbol_Public` in its `rust_symbols.go`), so the two tools
//! put a function on the same side of a declaration they both read.

use nomos_cap_naming_policy::Case;
use nomos_cap_syntax::PayloadItem;

/// The case a function name is judged against, one per side of visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FunctionCases
{
    /// For a function declared exactly `pub`.
    pub(super) exported: Case,
    /// For every other function.
    pub(super) unexported: Case,
}

impl FunctionCases
{
    /// One case for both sides: what a function in a language whose read does not split by
    /// visibility is judged against.
    #[must_use]
    pub(super) const fn Both(case: Case) -> Self
    {
        return Self { exported: case, unexported: case };
    }

    /// The case `function` is judged against: [`Self::exported`] when it declares itself public,
    /// [`Self::unexported`] otherwise.
    #[must_use]
    pub(super) fn For(self, function: &PayloadItem) -> Case
    {
        if function.Is_Public()
        {
            return self.exported;
        }

        return self.unexported;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_syntax::Observation;

    const SPLIT: FunctionCases = FunctionCases { exported: Case::UpperSnake, unexported: Case::LowerSnake };

    fn Function_Declared(visibility: &str) -> PayloadItem
    {
        return PayloadItem {
            ordinal: 0,
            kind: nomos_cap_syntax::FUNCTION.to_owned(),
            visibility: visibility.to_owned(),
            qualified_name: "as_str".to_owned(),
            documentation: Observation::NotObserved,
            shape: Observation::NotObserved,
        };
    }

    #[test]
    fn Test_For_Should_Judge_Only_A_Function_Declared_Exactly_Pub_As_Exported()
    {
        assert_eq!(SPLIT.For(&Function_Declared("Public")), Case::UpperSnake);

        for visibility in ["Private", "Restricted(crate)", "Restricted(super)", "Restricted(in some::place)", "NotApplicable"]
        {
            assert_eq!(SPLIT.For(&Function_Declared(visibility)), Case::LowerSnake, "{visibility}");
        }
    }

    #[test]
    fn Test_Both_Should_Judge_Either_Side_Of_Visibility_Against_One_Case()
    {
        let both = FunctionCases::Both(Case::LowerSnake);

        assert_eq!(both.For(&Function_Declared("Public")), Case::LowerSnake);
        assert_eq!(both.For(&Function_Declared("Private")), Case::LowerSnake);
    }
}
