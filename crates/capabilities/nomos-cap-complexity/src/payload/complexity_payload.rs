//! [`ComplexityPayload`], one file's answer.

use super::function_complexity::FunctionComplexity;
use super::metric_descriptor::MetricDescriptor;

/// One source file's answer: the descriptor its numbers are read under, and one entry per
/// function the file defines.
///
/// A file defining no function answers with the descriptor and no entry -- a real, examined
/// answer, and not the same fact as a file nobody measured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComplexityPayload
{
    /// How every value below may and may not be read.
    pub descriptor: MetricDescriptor,
    /// Every function the file defines, in the order they appear.
    pub functions: Vec<FunctionComplexity>,
}
