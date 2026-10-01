//! [`ComplexityReading`], and [`Read_Complexity`], which produces one.

use crate::walk::Walk;
use crate::ParseFailure;
use nomos_cap_complexity::FunctionComplexity;
use syn::visit::Visit;

/// The result of reading one file for its functions' complexity.
///
/// A file that parsed and defines no function, and a file that did not parse, are two different
/// answers, so neither is an empty list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComplexityReading
{
    /// Every function the file defines, in the order each one opens.
    Parsed(Vec<FunctionComplexity>),
    /// The file is not Rust `syn` can parse.
    Unparseable(ParseFailure),
}

/// Reads Rust source for each function's cyclomatic complexity.
///
/// Takes the text, not a path, so it has no way to reach a second file, which is what keeps
/// the guarantee's `IncrementalGranularity::File` true rather than asserted.
#[must_use]
pub fn Read_Complexity(source: &str) -> ComplexityReading
{
    let file = match syn::parse_file(source)
    {
        Ok(file) => file,
        Err(error) =>
        {
            let at = error.span().start();
            return ComplexityReading::Unparseable(ParseFailure { line: at.line, column: at.column, message: error.to_string() });
        }
    };

    let mut walk = Walk::New();
    walk.visit_file(&file);

    return ComplexityReading::Parsed(walk.functions);
}
