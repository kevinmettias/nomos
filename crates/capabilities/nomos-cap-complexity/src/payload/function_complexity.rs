//! [`FunctionComplexity`], one function's measured complexity.

/// One function's cyclomatic complexity, where the function is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionComplexity
{
    /// The function's name, qualified by the `impl` or `trait` it sits in where it has one, so
    /// two methods of one name in one file read apart.
    pub function: String,
    /// The one-based line its signature starts on.
    pub line: usize,
    /// One more than the number of decision points in its body.
    pub complexity: usize,
}
