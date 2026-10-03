//! The case a function's name is judged against, on each side of visibility, and in Go for a
//! method apart from a function.
//!
//! `OD-RULES-035` decision 7 has a Rust function read its case through the refinement of
//! `function` its visibility selects, so one source can hold names judged against two cases, and
//! decision 8 has every language `function-naming-convention` judges read the same way. Which side
//! a function is on is the syntax fact's own word for it: a function it labels public is exported,
//! and everything else is not. In Rust that is exactly `pub`, so private, `pub(crate)`,
//! `pub(super)`, `pub(in path)`, and a trait declaration's member, which declares no visibility of
//! its own, are unexported. code-standards draws its Rust line at exactly `pub` too
//! (`is_Symbol_Public` in its `rust_symbols.go`), and also requires the declaration to sit at the
//! file's top level, so a `pub` function inside an `impl` block or a nested module is exported
//! here and unexported there. No repository in reach declares two different cases for the two
//! sides of a Rust function, so no verdict differs; decision 8 records it and leaves it.
//!
//! Decision 8 also has a Go method read the method keys ahead of the function keys. A Go method is
//! told apart by [`Is_Go_Method`]; no other language's is, so [`SourceCases::methods`] is `None`
//! for every source not written in Go.

use nomos_cap_naming_policy::Case;
use nomos_cap_syntax::PayloadItem;

/// The case a function name is judged against, one per side of visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FunctionCases
{
    /// For a function the syntax fact labels public: in Rust, one declared exactly `pub`.
    pub(super) exported: Case,
    /// For every other function.
    pub(super) unexported: Case,
}

/// What one source's functions are judged against: a function's cases, and for a Go source a
/// method's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SourceCases
{
    /// For a function that is not a method, and for every function in a language whose methods
    /// the syntax fact does not tell apart.
    pub(super) functions: FunctionCases,
    /// For a Go method; `None` for a source in any other language.
    pub(super) methods: Option<FunctionCases>,
}

impl SourceCases
{
    /// The cases a source with no methods told apart is judged against.
    #[cfg(test)]
    #[must_use]
    pub(super) const fn Functions(functions: FunctionCases) -> Self
    {
        return Self { functions, methods: None };
    }

    /// The case `function` is judged against: a method's when this source tells methods apart and
    /// `function` is one, a function's otherwise, on the side of visibility it declares.
    #[must_use]
    pub(super) fn For(self, function: &PayloadItem) -> Case
    {
        let sides = match self.methods
        {
            Some(methods) if Is_Go_Method(function) => methods,
            _ => self.functions,
        };

        return sides.For(function);
    }
}

/// Whether a Go function is a method.
///
/// A Go function is declared at a package's top level and nests under nothing but the receiver it
/// is declared on, which the syntax fact writes as the qualifier of its name (`Table::Rows`), so a
/// qualified Go function is a method and an unqualified one is not. That is code-standards' line
/// too: its Go front end makes a function with a receiver a method. Only for a Go source: a Rust
/// function is qualified by a module as readily as by an `impl` block.
#[must_use]
pub(super) fn Is_Go_Method(function: &PayloadItem) -> bool
{
    return function.qualified_name.contains("::");
}

impl FunctionCases
{
    /// One case for both sides of visibility.
    #[cfg(test)]
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

    /// A Go method is the one function the syntax fact qualifies, by its receiver.
    #[test]
    fn Test_Is_Go_Method_Should_Hold_For_A_Function_Qualified_By_Its_Receiver_Only()
    {
        let mut method = Function_Declared("Public");
        method.qualified_name = "Table::Rows".to_owned();

        assert!(Is_Go_Method(&method));
        assert!(!Is_Go_Method(&Function_Declared("Public")));
    }

    /// A source that tells methods apart judges a method against the method's case on its own side
    /// of visibility, and a free function against the function's; one that does not judges both
    /// against the function's, however the name is qualified.
    #[test]
    fn Test_For_Should_Judge_A_Method_Against_A_Methods_Case_Only_Where_Methods_Are_Told_Apart()
    {
        let methods = FunctionCases { exported: Case::UpperCamel, unexported: Case::LowerCamel };
        let mut method = Function_Declared("Private");
        method.qualified_name = "Table::rows".to_owned();

        let go = SourceCases { functions: SPLIT, methods: Some(methods) };
        assert_eq!(go.For(&method), Case::LowerCamel);
        assert_eq!(go.For(&Function_Declared("Private")), Case::LowerSnake);

        assert_eq!(SourceCases::Functions(SPLIT).For(&method), Case::LowerSnake);
    }
}
