//! [`ConditionalPayload`], one file's answer.

use super::build_selection::BuildSelection;
use super::conditional_region::ConditionalRegion;
use super::file_definition::FileDefinition;

/// One C# file's answer: the build it is about, the symbols that build defines, the definitions
/// the file adds or removes itself, and every branch of every conditional chain in the file.
///
/// A file with no conditional directive answers with no region -- a real, examined answer, and
/// not the same fact as a file nobody examined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionalPayload
{
    /// The build every state below is judged under.
    pub selection: BuildSelection,
    /// Every preprocessor symbol the build defines before the file's own directives, sorted.
    pub symbols: Vec<String>,
    /// Every `#define` and `#undef` in a compiled part of the file, in line order.
    pub definitions: Vec<FileDefinition>,
    /// Every branch of every conditional chain, in the order its directive appears.
    pub regions: Vec<ConditionalRegion>,
}
